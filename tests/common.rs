use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use assert_cmd::Command;
use sha2::{Digest, Sha256};
use std::time::Duration;
use std::{fs, io, thread};
use tempfile::Builder;

pub fn make_tmp(filename: &str) -> io::Result<PathBuf> {
    let temp_dir = Builder::new().tempdir()?;
    let n_bytes = 1000;
    let data: Vec<u8> = (0..n_bytes).map(|_| rand::random::<u8>()).collect();
    let file_path = temp_dir.path().join(filename);
    let mut file = File::create(&file_path)?;
    file.write_all(&data)?;
    let dir_path = temp_dir.keep();
    Ok(dir_path)
}

pub fn compare_files(f1: &PathBuf, f2: &PathBuf) -> io::Result<bool> {
    let data1 = fs::read(f1)?;
    let data2 = fs::read(f2)?;
    let h1 = Sha256::digest(&data1);
    let h2 = Sha256::digest(&data2);
    Ok(h1 == h2)
}

/// `proto` is the protocol flag name as used by the CLI (e.g. "http", "ftp", "tftp").
pub fn test_server_e2e(
    proto: &str,
    port: u16,
    dl_cmd: String,
    file_in: &str,
    file_out: &str,
) -> Result<bool, String> {
    let dir_path = make_tmp(file_in)
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;
    let dir_path_c = dir_path.clone();
    let proto = proto.to_string();

    let server = thread::spawn(move || {
        let mut cmd = Command::cargo_bin("quick-serve").unwrap();
        let arg_str = format!(
            "--headless -d={} -b=127.0.0.1 -v --{}={}",
            dir_path.to_str().unwrap(),
            proto,
            port
        );
        println!("Running cmd: {}", arg_str);
        cmd.timeout(Duration::from_secs(2));
        cmd.args(arg_str.split_whitespace());
        cmd.unwrap()
    });

    let client = thread::spawn(move || {
        thread::sleep(Duration::from_millis(700));
        let mut cmd = Command::new("sh");
        cmd.timeout(Duration::from_secs(3));
        cmd.arg("-c");
        cmd.arg(&dl_cmd);
        cmd.env("PATH", "/bin");
        cmd.unwrap()
    });

    let out_client = client.join();
    if out_client.is_err() {
        return Err(format!("Download failed: {:?}", out_client));
    }

    // The result here is always an error as the server gets killed.
    let out_server = server.join();
    if out_server.is_ok() {
        return Err(format!(
            "Server exited gracefully while it should have not: {:?}",
            out_server
        ));
    }

    let file_in = dir_path_c.join(file_in);
    if !file_in.exists() {
        return Err(format!("Source file {} does not exist!", file_in.to_str().unwrap()));
    }

    let file_out = PathBuf::from(file_out);
    if !file_out.exists() {
        return Err(format!("Output file {} does not exist!", file_out.to_string_lossy()));
    }

    let metadata = file_out.metadata()
        .map_err(|e| format!("Failed to read metadata: {}", e))?;
    if metadata.len() == 0 {
        return Err(format!("File {} is empty!", file_out.to_string_lossy()));
    }

    let files_match = compare_files(&file_in, &file_out)
        .map_err(|e| format!("Failed to compare files: {}", e))?;

    if !files_match {
        return Err("Content of files served and downloaded are not the same!".to_string());
    }

    Ok(true)
}
