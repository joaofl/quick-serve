mod common;

use common::test_server_e2e;

#[test]
fn test_file_download_success() {
    let port = 8079u16;
    let file_in = "data.bin";
    let file_out = "/tmp/data-out-http.bin";
    let dl_cmd = format!("wget -t2 -T1 http://127.0.0.1:{}/{} -O {}", port, file_in, file_out);
    let result = test_server_e2e("http", port, dl_cmd, file_in, file_out);
    assert!(result.is_ok(), "Test failed: {:?}", result.err());
}

#[test]
fn test_file_not_found() {
    let port = 8080u16;
    let file_in = "data.bin";
    let file_out = "/tmp/data-out-http-404.bin";
    let dl_cmd = format!("wget -t1 -T1 http://127.0.0.1:{}/nonexistent.bin -O {} 2>&1 || true", port, file_out);
    let result = test_server_e2e("http", port, dl_cmd, file_in, file_out);
    assert!(result.is_err(), "Expected failure for non-existent file");
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("empty"), "Expected empty file error, got: {}", err_msg);
}

#[test]
fn test_path_is_directory() {
    let port = 8081u16;
    let file_in = "data.bin";
    let file_out = "/tmp/data-out-http-dir.bin";
    let dl_cmd = format!("wget -t1 -T1 http://127.0.0.1:{}/ -O {} 2>&1 || true", port, file_out);
    let result = test_server_e2e("http", port, dl_cmd, file_in, file_out);
    assert!(result.is_err(), "Expected failure for directory path");
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("empty") || err_msg.contains("does not exist"),
        "Expected empty file or non-existent error, got: {}", err_msg);
}

#[test]
fn test_path_traversal_blocked() {
    let port = 8082u16;
    let file_in = "data.bin";
    let file_out = "/tmp/data-out-http-traversal.bin";
    let dl_cmd = format!("wget -t1 -T1 http://127.0.0.1:{}/../../etc/passwd -O {} 2>&1 || true", port, file_out);
    let result = test_server_e2e("http", port, dl_cmd, file_in, file_out);
    assert!(result.is_err(), "Expected failure for path traversal attempt");
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("empty") || err_msg.contains("does not exist"),
        "Expected empty file or non-existent error, got: {}", err_msg);
}
