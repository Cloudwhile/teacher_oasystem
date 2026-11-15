use std::{
    collections::VecDeque,
    io::Read,
    sync::{Arc, Mutex},
};

const MAX_LINES: usize = 1500;
const MAX_BYTES: usize = 512 * 1024;
const MAX_CAPTURE_CHUNK: usize = 64 * 1024;
pub type SharedLogs = Arc<Mutex<LogBuffer>>;

#[derive(Default)]
pub struct LogBuffer {
    lines: VecDeque<String>,
    generation: u64,
    bytes: usize,
    total_pushed: u64,
    cleared_at: u64,
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
        self.total_pushed = self.total_pushed.saturating_add(1);
        self.generation = self.generation.wrapping_add(1);
    }
    pub fn clear(&mut self) {
        self.lines.clear();
        self.bytes = 0;
        self.total_pushed = 0;
        self.cleared_at = 0;
        self.generation = self.generation.wrapping_add(1);
    }
    pub fn clear_view(&mut self) {
        self.cleared_at = self.total_pushed;
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
    pub fn view_text(&self, filter: &str) -> String {
        let retained = u64::try_from(self.lines.len()).unwrap_or(u64::MAX);
        let first_index = self.total_pushed.saturating_sub(retained);
        let hidden = usize::try_from(self.cleared_at.saturating_sub(first_index))
            .unwrap_or(self.lines.len())
            .min(self.lines.len());
        let filter = filter.to_lowercase();
        self.lines
            .iter()
            .skip(hidden)
            .filter(|line| line.to_lowercase().contains(&filter))
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub fn filter_lines(text: &str, filter: &str) -> String {
    let filter = filter.to_lowercase();
    if filter.is_empty() {
        return text.to_string();
    }
    text.lines()
        .filter(|line| line.to_lowercase().contains(&filter))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn record(logs: &SharedLogs, message: &str) {
    if let Ok(mut logs) = logs.lock() {
        logs.push(format!(
            "{}  {message}",
            chrono::Local::now().format("%H:%M:%S")
        ));
    }
}

pub fn record_raw(logs: &SharedLogs, line: &str) {
    if let Ok(mut logs) = logs.lock() {
        logs.push(line.to_string());
    }
}

pub fn capture(reader: impl Read + Send + 'static, logs: SharedLogs, token: String) {
    std::thread::spawn(move || {
        capture_stream(reader, &logs, &token, MAX_CAPTURE_CHUNK);
    });
}

fn capture_stream(mut reader: impl Read, logs: &SharedLogs, token: &str, max_chunk: usize) {
    let effective_max = max_chunk.max(token.len().saturating_mul(2).saturating_add(32));
    let holdback = token.len().saturating_sub(1);
    let mut pending = Vec::new();
    let mut buffer = [0_u8; 4096];

    loop {
        let read_len = match reader.read(&mut buffer) {
            Ok(0) | Err(_) => 0,
            Ok(n) => n,
        };
        if read_len == 0 {
            if !pending.is_empty() {
                let trimmed = trim_line_end(&pending);
                record_raw(logs, &clean(trimmed, token));
            }
            break;
        }

        pending.extend_from_slice(&buffer[..read_len]);
        loop {
            if let Some(newline_pos) = pending.iter().position(|&byte| byte == b'\n') {
                let line = trim_line_end(&pending[..newline_pos]);
                record_raw(logs, &clean(line, token));
                pending.drain(..=newline_pos);
                continue;
            }
            if pending.len() < effective_max {
                break;
            }
            let split = safe_chunk_split(&pending, token.as_bytes(), holdback);
            let chunk = trim_line_end(&pending[..split]);
            record_raw(logs, &clean(chunk, token));
            pending.drain(..split);
        }
    }
}

fn trim_line_end(mut bytes: &[u8]) -> &[u8] {
    while let Some((&last, rest)) = bytes.split_last() {
        if last == b'\n' || last == b'\r' {
            bytes = rest;
        } else {
            break;
        }
    }
    bytes
}

fn safe_chunk_split(pending: &[u8], token: &[u8], holdback: usize) -> usize {
    let mut cut = pending.len().saturating_sub(holdback);
    if !token.is_empty() {
        let window_start = cut.saturating_sub(holdback);
        if let Some(offset) = pending[window_start..]
            .windows(token.len())
            .position(|window| window == token)
        {
            let token_start = window_start + offset;
            if token_start > 0 && token_start < cut {
                cut = token_start;
            }
        }
    }
    // Avoid splitting an incomplete trailing CSI escape sequence (`\x1b[...`).
    let esc_scan_start = cut.saturating_sub(16);
    if let Some(rel_esc) = pending[esc_scan_start..cut]
        .iter()
        .rposition(|&byte| byte == 0x1b)
    {
        let esc_pos = esc_scan_start + rel_esc;
        let has_terminator = pending[esc_pos + 1..cut]
            .iter()
            .any(|&byte| (0x40..=0x7e).contains(&byte) && byte != b'[');
        if !has_terminator && esc_pos > 0 {
            cut = esc_pos;
        }
    }
    // Avoid splitting a multi-byte UTF-8 scalar value across chunks.
    match std::str::from_utf8(&pending[..cut]) {
        Ok(_) => cut,
        Err(error) if error.error_len().is_none() && error.valid_up_to() > 0 => error.valid_up_to(),
        Err(_) => {
            let tail_start = cut.saturating_sub(3);
            if let Err(tail_err) = std::str::from_utf8(&pending[tail_start..cut])
                && tail_err.error_len().is_none()
                && tail_start + tail_err.valid_up_to() > 0
            {
                tail_start + tail_err.valid_up_to()
            } else {
                cut
            }
        }
    }
}

fn clean(bytes: &[u8], token: &str) -> String {
    let stripped = String::from_utf8_lossy(&strip_ansi_escapes::strip(bytes)).into_owned();
    if token.is_empty() {
        stripped
    } else {
        stripped.replace(token, "[已隐藏]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    struct TinyChunkReader<'a> {
        data: &'a [u8],
        step: usize,
    }

    impl Read for TinyChunkReader<'_> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if self.data.is_empty() {
                return Ok(0);
            }
            let count = self.data.len().min(self.step).min(buf.len());
            buf[..count].copy_from_slice(&self.data[..count]);
            self.data = &self.data[count..];
            Ok(count)
        }
    }

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

    #[test]
    fn clear_view_hides_existing_lines_without_erasing_export_history() {
        let mut logs = LogBuffer::default();
        logs.push("line-1".into());
        logs.push("line-2".into());
        logs.clear_view();

        assert_eq!(logs.view_text(""), "");
        assert_eq!(logs.text(""), "line-1\nline-2");

        logs.push("line-3 error".into());
        assert_eq!(logs.view_text(""), "line-3 error");
        assert_eq!(logs.view_text("ERROR"), "line-3 error");
        assert_eq!(logs.text(""), "line-1\nline-2\nline-3 error");
    }

    #[test]
    fn capture_preserves_json_lines_without_double_timestamp() {
        let logs = SharedLogs::default();
        let payload =
            b"{\"timestamp\":\"2026-11-15T08:00:00Z\",\"level\":\"INFO\",\"token\":\"cap-123\"}\r\n";
        capture_stream(Cursor::new(payload), &logs, "cap-123", 1024);

        let captured = logs.lock().unwrap().text("");
        assert_eq!(
            captured,
            "{\"timestamp\":\"2026-11-15T08:00:00Z\",\"level\":\"INFO\",\"token\":\"[已隐藏]\"}"
        );
    }

    #[test]
    fn capture_preserves_utf8_and_redacts_tokens_across_forced_splits() {
        let logs = SharedLogs::default();
        let token = "secret-mgmt-token";
        let raw = format!(
            "教师成果管理系统日志验证教师成果管理系统日志验证-{token}-教师成果管理系统日志验证结束"
        );
        let reader = TinyChunkReader {
            data: raw.as_bytes(),
            step: 5,
        };
        capture_stream(reader, &logs, token, 24);

        let joined = logs.lock().unwrap().text("").replace('\n', "");
        assert!(
            !joined.contains('\u{fffd}'),
            "must not corrupt UTF-8: {joined}"
        );
        assert!(!joined.contains(token), "must redact split token: {joined}");
        assert_eq!(
            joined,
            "教师成果管理系统日志验证教师成果管理系统日志验证-[已隐藏]-教师成果管理系统日志验证结束"
        );
    }
}
