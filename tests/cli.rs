use predicates::prelude::*;
use assert_cmd::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;

// ── Help / version ────────────────────────────────────────────────────────────

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("quick-serve").unwrap();
    cmd.arg("--help");
    cmd.assert().success().stdout(predicate::str::contains("Usage: quick-serve"));
}

#[test]
fn test_gui_cli_help() {
    let mut cmd = Command::cargo_bin("quick-serve-gui").unwrap();
    cmd.arg("--help");
    cmd.assert().success().stdout(predicate::str::contains("Usage: quick-serve"));
}

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("quick-serve").unwrap();
    cmd.arg("--version");
    cmd.assert().success().stdout(predicate::str::contains("quick-serve"));
}

// ── Error handling ────────────────────────────────────────────────────────────

#[test]
fn test_unknown_flag_rejected() {
    let mut cmd = Command::cargo_bin("quick-serve").unwrap();
    cmd.arg("--this-flag-does-not-exist");
    cmd.assert().failure().stderr(predicate::str::contains("unexpected argument"));
}

#[test]
fn test_no_server_specified_exits_with_message() {
    let mut cmd = Command::cargo_bin("quick-serve").unwrap();
    cmd.arg("--headless");
    cmd.assert()
        .code(2)
        .stdout(predicate::str::contains("No server specified"));
}

// ── Server startup (spawned with short timeout, output captured manually) ────

/// Spawns quick-serve with the given args, waits briefly for startup log lines
/// to appear, then kills the process and returns stdout.
fn capture_startup_output(args: &[&str]) -> String {
    let bin = std::env::var("CARGO_BIN_EXE_quick-serve")
        .unwrap_or_else(|_| "target/debug/quick-serve".into());

    let mut child = std::process::Command::new(&bin)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn quick-serve");

    thread::sleep(Duration::from_millis(400));
    child.kill().ok();
    let output = child.wait_with_output().unwrap();
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn test_http_server_starts_on_specified_port() {
    let stdout = capture_startup_output(&["--headless", "--http=17801"]);
    assert!(stdout.contains("http") && stdout.contains("17801"),
        "Expected HTTP server on port 17801 in output:\n{}", stdout);
}

#[test]
fn test_ftp_server_starts_on_specified_port() {
    let stdout = capture_startup_output(&["--headless", "--ftp=17802"]);
    assert!(stdout.contains("ftp") && stdout.contains("17802"),
        "Expected FTP server on port 17802 in output:\n{}", stdout);
}

#[test]
fn test_tftp_server_starts_on_specified_port() {
    let stdout = capture_startup_output(&["--headless", "--tftp=17803"]);
    assert!(stdout.contains("tftp") && stdout.contains("17803"),
        "Expected TFTP server on port 17803 in output:\n{}", stdout);
}

#[test]
fn test_multiple_servers_start_together() {
    let stdout = capture_startup_output(&["--headless", "--http=17804", "--ftp=17805"]);
    assert!(stdout.contains("17804"), "Expected HTTP port 17804 in output:\n{}", stdout);
    assert!(stdout.contains("17805"), "Expected FTP port 17805 in output:\n{}", stdout);
}

#[test]
fn test_custom_bind_ip_used() {
    let stdout = capture_startup_output(&["--headless", "--http=17806", "--bind-ip=127.0.0.1"]);
    assert!(stdout.contains("127.0.0.1"),
        "Expected bind IP in output:\n{}", stdout);
}

#[test]
fn test_verbose_flag_enables_debug_logs() {
    let stdout = capture_startup_output(&["--headless", "--http=17807", "-v"]);
    assert!(stdout.contains("DEBUG") || stdout.contains("debug") || stdout.contains("Spawn") || stdout.contains("spawn"),
        "Expected debug-level output with -v flag:\n{}", stdout);
}
