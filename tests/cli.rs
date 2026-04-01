use predicates::prelude::*;
use assert_cmd::Command;

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
