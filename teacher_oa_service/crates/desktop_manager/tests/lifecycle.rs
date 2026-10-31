use management_protocol::SaveConfig;
use std::{
    net::TcpListener,
    time::{Duration, Instant},
};
use teacher_oa_desktop::{process::Supervisor, settings::LauncherSettings};

struct Managed(Supervisor);
impl Drop for Managed {
    fn drop(&mut self) {
        if self.0.pid().is_some() {
            let _ = self.0.force_stop();
        }
    }
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn wait_for(server: &mut Supervisor, running: bool) {
    let deadline = Instant::now() + Duration::from_secs(40);
    loop {
        let result = server.poll();
        if running && server.connected || !running && server.pid().is_none() {
            return;
        }
        assert!(
            result.is_ok(),
            "{result:?}\n{}",
            server.logs.lock().unwrap().text("")
        );
        assert!(
            Instant::now() < deadline,
            "timed out\n{}",
            server.logs.lock().unwrap().text("")
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
#[ignore = "requires TEACHER_OA_TEST_SERVICE pointing to a locally built service executable"]
fn real_service_starts_saves_restarts_and_stops_through_api() {
    assert!(
        !std::env::vars_os().any(|(key, _)| key.to_string_lossy().starts_with("ASTER__")),
        "run isolated tests without inherited ASTER__ overrides"
    );
    let executable = std::env::var("TEACHER_OA_TEST_SERVICE").expect("test service path");
    let directory = tempfile::tempdir().unwrap();
    let port = free_port();
    let path = directory.path().join("isolated.toml");
    std::fs::write(
        &path,
        format!(
            r#"
[server]
host = "127.0.0.1"
port = {port}
workers = 1
[database]
url = "sqlite://isolated.db?mode=rwc"
[storage]
backend = "memory"
[auth]
environment = "test"
cookie_secure = false
public_origin = "http://127.0.0.1:{port}"
[logging]
level = "info"
format = "text"
"#
        ),
    )
    .unwrap();
    let mut managed = Managed(Supervisor::new(LauncherSettings {
        executable,
        working_directory: directory.path().to_string_lossy().into_owned(),
        config_path: path.to_string_lossy().into_owned(),
        base_url: format!("http://127.0.0.1:{port}"),
    }));
    managed.0.start().unwrap();
    assert!(managed.0.start().is_err(), "must reject duplicate starts");
    wait_for(&mut managed.0, true);
    let status = managed.0.last_status.as_ref().unwrap();
    assert_eq!(Some(status.pid), managed.0.pid());
    assert!(status.dependencies.iter().all(|row| row.healthy));
    let snapshot = managed.0.config().unwrap();
    let mut value = SaveConfig {
        config: snapshot.saved,
        revision: snapshot.revision,
    };
    let next_port = free_port();
    value.config.port = next_port;
    value.config.log_level = "warn".into();
    let saved = managed.0.save_config(&value).unwrap();
    assert!(saved.restart_required);
    assert_eq!(saved.active.port, port);
    assert_eq!(saved.saved.port, next_port);
    managed.0.stop().unwrap();
    wait_for(&mut managed.0, false);
    managed.0.settings.base_url = format!("http://127.0.0.1:{next_port}");
    managed.0.start().unwrap();
    wait_for(&mut managed.0, true);
    assert_eq!(managed.0.config().unwrap().active.port, next_port);
    managed.0.stop().unwrap();
    wait_for(&mut managed.0, false);
    assert!(!managed.0.logs.lock().unwrap().text("").is_empty());
}
