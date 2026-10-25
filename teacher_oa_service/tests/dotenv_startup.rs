use aster_forge_test::temp::TestTempDir;
use std::{
    fs,
    process::{Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

fn directory() -> TestTempDir {
    TestTempDir::new_in(
        std::env::current_dir()
            .expect("cwd")
            .join("target/dotenv-tests"),
        "dotenv-startup",
    )
}

fn run(directory: &TestTempDir, override_path: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_teacher_oa_service"));
    command
        .current_dir(directory.path())
        .env_remove("ASTER_CONFIG")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(path) = override_path {
        command.env("ASTER_CONFIG", path);
    }
    let mut child = command.spawn().expect("start process");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if child.try_wait().expect("process status").is_some() {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("expected configuration error before the service starts");
        }
        thread::sleep(Duration::from_millis(20));
    }
    child.wait_with_output().expect("process output")
}

#[test]
fn dotenv_selects_configuration_without_environment_injection() {
    let directory = directory();
    fs::write(
        directory.join(".env"),
        "ASTER_CONFIG=dotenv-selected.toml\n",
    )
    .expect("dotenv");
    fs::write(directory.join("dotenv-selected.toml"), "[server\n").expect("invalid config");
    let output = run(&directory, None);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("dotenv-selected"));
}

#[test]
fn process_configuration_takes_precedence_over_dotenv() {
    let directory = directory();
    fs::write(
        directory.join(".env"),
        "ASTER_CONFIG=dotenv-selected.toml\n",
    )
    .expect("dotenv");
    fs::write(directory.join("dotenv-selected.toml"), "[server\n").expect("dotenv config");
    fs::write(directory.join("process-selected.toml"), "[server\n").expect("process config");
    let output = run(&directory, Some("process-selected.toml"));
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(error.contains("process-selected"));
    assert!(!error.contains("dotenv-selected"));
}

#[test]
fn malformed_dotenv_stops_startup_without_printing_secret_values() {
    let directory = directory();
    let secret = "private-test-secret-that-must-not-be-logged";
    fs::write(
        directory.join(".env"),
        format!("WECOM_CORP_SECRET=\"{secret}"),
    )
    .expect("invalid dotenv");
    let output = run(&directory, None);
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains(".env"));
    assert!(!error.contains(secret));
    assert!(!String::from_utf8_lossy(&output.stdout).contains(secret));
}
