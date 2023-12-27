
#[cfg(test)]
pub mod test_server {
    use log::{info};
    use async_process::Command;
    use crate::servers::{Server, FTPRunner};
    use std::fs::{File};
    use std::io::{Write};
    use std::{sync::Arc, path::PathBuf};
    use std::io::Error;
    use std::ops::Deref;
    use tokio::time::{self, Duration};
    use std::string::String;

    async fn run_cmd(cmd: &String) -> Result<String, String> {
        info!("Running command: {}", cmd);

        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output().await
            .expect("failed to execute process");

        // let out = String::from_utf8_lossy(&output.stdout).deref().to_string();
        let err = String::from_utf8_lossy(&output.stderr).deref().to_string();
        // print!("{}", err);
        if output.status.success() { Ok(err) } else { Err(err) }
    }

    pub async fn mkfile() -> Result<(PathBuf, String), Error> {
        // Create a temporary directory
        let temp_dir_path = PathBuf::from("/tmp/any_serve");

        let cmd = &format!("mkdir --parents --verbose {}", &temp_dir_path.to_string_lossy());
        let _r = run_cmd(cmd).await;

        // Create a file inside the temporary directory
        let file_name = String::from("in.txt");
        let file_path = temp_dir_path.join(file_name.clone());
        let mut file = File::create(&file_path)?;

        // Write some data to the file
        let some_text = "Hello, this is some known data written to the file!";
        write!(file, "{}\n", some_text.repeat(100))?;

        info!("Temporary directory: {:?}", temp_dir_path);
        info!("File path: {:?}", file_path);

        Ok((temp_dir_path, file_name))
    }

    pub async fn test_server_e2e(s: Arc<Server>, cmd: String) {
        let sc = s.clone();
        let runner = tokio::spawn(async move {
            sc.deref().runner().await;
        });

        time::sleep(Duration::from_millis(100)).await;
        let _ = s.start();
        let r = run_cmd(&cmd).await;
        assert!(r.is_ok(), "Failed to download with error: {}", r.unwrap());

        time::sleep(Duration::from_millis(100)).await;
        let _ = s.terminate();
        let r = run_cmd(&cmd).await;
        assert!(r.is_err(), "Succeed to download. Server not terminated");

        let _ = runner.await;
    }
}

