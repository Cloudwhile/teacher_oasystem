use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Read},
    sync::{Arc, Mutex},
};

const MAX_LINES: usize = 1500;
const MAX_BYTES: usize = 512 * 1024;
pub type SharedLogs = Arc<Mutex<LogBuffer>>;

#[derive(Default)]
pub struct LogBuffer {
    lines: VecDeque<String>,
    generation: u64,
    bytes: usize,
}

impl LogBuffer {
    pub fn push(&mut self, line: String) {
        self.bytes += line.len();
        while self.lines.len() >= MAX_LINES || self.bytes > MAX_BYTES {
            if let Some(old) = self.lines.pop_front() {
                self.bytes -= old.len();
            } else {
                break;
            }
        }
        self.lines.push_back(line);
        self.generation = self.generation.wrapping_add(1);
    }
    pub fn clear(&mut self) {
        self.lines.clear();
        self.bytes = 0;
        self.generation = self.generation.wrapping_add(1);
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn text(&self, filter: &str) -> String {
        let filter = filter.to_lowercase();
        self.lines
            .iter()
            .filter(|line| line.to_lowercase().contains(&filter))
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub fn record(logs: &SharedLogs, message: &str) {
    if let Ok(mut logs) = logs.lock() {
        logs.push(format!(
            "{}  {message}",
            chrono::Local::now().format("%H:%M:%S")
        ));
    }
}

pub fn capture(reader: impl Read + Send + 'static, logs: SharedLogs, token: String) {
    std::thread::spawn(move || {
        let mut reader = BufReader::new(reader);
        loop {
            // Bound each chunk even when a child writes an arbitrarily long line.
            let mut bytes = Vec::new();
            match reader.by_ref().take(8192).read_until(b'\n', &mut bytes) {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    let text = clean(&bytes, &token);
                    record(&logs, text.trim_end());
                }
            }
        }
    });
}

fn clean(bytes: &[u8], token: &str) -> String {
    String::from_utf8_lossy(&strip_ansi_escapes::strip(bytes)).replace(token, "[已隐藏]")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_log_text_has_no_terminal_escapes_or_management_token() {
        assert_eq!(
            clean(
                b"\x1b[32mINFO\x1b[0m secret-capability",
                "secret-capability"
            ),
            "INFO [已隐藏]"
        );
    }
    #[test]
    fn logs_are_bounded_and_filter_does_not_destroy_history() {
        let mut logs = LogBuffer::default();
        for n in 0..1600 {
            logs.push(format!("INFO {n}"));
        }
        assert_eq!(logs.lines.len(), 1500);
        assert_eq!(logs.lines.front().unwrap(), "INFO 100");
        assert_eq!(logs.text("1599"), "INFO 1599");
        assert_eq!(logs.lines.len(), 1500);
        logs.clear();
        assert!(logs.text("").is_empty());
    }
}
