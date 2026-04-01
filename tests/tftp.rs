mod common;

use common::test_server_e2e;

#[test]
fn test_file_download_success() {
    let port = 6966u16;
    let file_in = "data.bin";
    let file_out = "/tmp/data-out-tftp.bin";
    let dl_cmd = format!("tftp 127.0.0.1 {} -m binary -c get {} {}", port, file_in, file_out);
    let result = test_server_e2e("tftp", port, dl_cmd, file_in, file_out);
    assert!(result.is_ok(), "Test failed: {:?}", result.err());
}

#[test]
fn test_file_not_found() {
    let port = 6967u16;
    let file_in = "data.bin";
    let file_out = "/tmp/data-out-tftp-404.bin";
    let dl_cmd = format!("tftp 127.0.0.1 {} -m binary -c get nonexistent.bin {} 2>&1 || true", port, file_out);
    let result = test_server_e2e("tftp", port, dl_cmd, file_in, file_out);
    assert!(result.is_err(), "Expected failure for non-existent file");
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("does not exist") || err_msg.contains("empty"),
        "Expected file not found error, got: {}", err_msg);
}

#[test]
fn test_path_is_directory() {
    let port = 6968u16;
    let file_in = "data.bin";
    let file_out = "/tmp/data-out-tftp-dir.bin";
    let dl_cmd = format!("tftp 127.0.0.1 {} -m binary -c get '' {} 2>&1 || true", port, file_out);
    let result = test_server_e2e("tftp", port, dl_cmd, file_in, file_out);
    assert!(result.is_err(), "Expected failure for directory path");
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("does not exist") || err_msg.contains("empty"),
        "Expected file not found or empty file error, got: {}", err_msg);
}

#[test]
fn test_path_traversal_blocked() {
    let port = 6969u16;
    let file_in = "data.bin";
    let file_out = "/tmp/data-out-tftp-traversal.bin";
    let dl_cmd = format!("tftp 127.0.0.1 {} -m binary -c get ../../etc/passwd {} 2>&1 || true", port, file_out);
    let result = test_server_e2e("tftp", port, dl_cmd, file_in, file_out);
    assert!(result.is_err(), "Expected failure for path traversal attempt");
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("does not exist") || err_msg.contains("empty"),
        "Expected file not found or empty file error, got: {}", err_msg);
}
