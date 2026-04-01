mod common;

use common::test_server_e2e;

#[test]
fn test_file_download_success() {
    let port = 2223u16;
    let file_in = "data.bin";
    let file_out = "/tmp/data-out-ftp.bin";
    let dl_cmd = format!("curl --retry 2 --retry-delay 1 ftp://127.0.0.1:{}/{} -o {}", port, file_in, file_out);
    let result = test_server_e2e("ftp", port, dl_cmd, file_in, file_out);
    assert!(result.is_ok(), "Test failed: {:?}", result.err());
}

#[test]
fn test_file_not_found() {
    let port = 2224u16;
    let file_in = "data.bin";
    let file_out = "/tmp/data-out-ftp-404.bin";
    let dl_cmd = format!("curl --retry 1 --retry-delay 1 ftp://127.0.0.1:{}/nonexistent.bin -o {} 2>&1 || true", port, file_out);
    let result = test_server_e2e("ftp", port, dl_cmd, file_in, file_out);
    assert!(result.is_err(), "Expected failure for non-existent file");
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("does not exist") || err_msg.contains("empty"),
        "Expected file not found error, got: {}", err_msg);
}

#[test]
fn test_path_is_directory() {
    let port = 2225u16;
    let file_in = "data.bin";
    let file_out = "/tmp/data-out-ftp-dir.bin";
    let dl_cmd = format!("curl --retry 1 --retry-delay 1 ftp://127.0.0.1:{}/nonexistent_subdir/file.txt -o {} 2>&1 || true", port, file_out);
    let result = test_server_e2e("ftp", port, dl_cmd, file_in, file_out);
    assert!(result.is_err(), "Expected failure for non-existent directory path");
}

#[test]
fn test_path_traversal_blocked() {
    let port = 2226u16;
    let file_in = "data.bin";
    let file_out = "/tmp/data-out-ftp-traversal.bin";
    let dl_cmd = format!("curl --retry 1 --retry-delay 1 ftp://127.0.0.1:{}/../../etc/passwd -o {} 2>&1 || true", port, file_out);
    let result = test_server_e2e("ftp", port, dl_cmd, file_in, file_out);
    assert!(result.is_err(), "Expected failure for path traversal attempt");
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("does not exist") || err_msg.contains("empty"),
        "Expected file not found or empty file error, got: {}", err_msg);
}
