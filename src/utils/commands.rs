#[cfg(test)]
pub mod wget {
    use tokio::process::Command as AsyncCommand;

    pub async fn download (url: String) -> i32 {
        let cmd = AsyncCommand::new("wget")
            .arg("--timeout=1")
            .arg("--tries=1")
            .arg("--output-document=/tmp/file-recv.txt")
            .arg(url)
            .output()
            .await.expect("Failed to execute command");

        return cmd.status.code().unwrap();
    }
}
