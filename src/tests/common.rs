
#[cfg(test)]
pub mod test_server {

    use std::sync::Arc;
    use tokio::time::{self, Duration};
    use crate::utils::commands::wget;
    use crate::servers::{HTTPServerRunner, FTPServerRunner, Protocol};
    use crate::servers::Server;
    // use crate::servers::common::ServerTrait;

    use std::fs::File;
    use std::io::prelude::*;
    use std::ops::Deref;

    pub async fn e2e(new_server: Server, port: u16) -> (i32, i32, i32, i32) {
        let server = Arc::new(new_server);
        let server_c = server.clone();

        let bind_address = "127.0.0.1".to_string();
        let url = format!("{}://{}:{}/file.txt", server.protocol.to_string(), bind_address.clone(), port);

        let temp_dir = tempfile::tempdir()
            .expect("Failed to create temp directory");
        let path = temp_dir.path().to_path_buf();

        // Create a temporary file inside the directory
        let mut temp_file = File::create(path.join("file.txt"))
            .expect("Failed to create temp file");

        temp_file.write_all(b"This is a temporary file!")
            .expect("Failed to write to temp file");

        let task_runner = tokio::spawn(async move {
            if server.protocol == Protocol::Ftp {
                FTPServerRunner::runner(server.deref()).await
            }
            else if server.protocol == Protocol::Http {
                HTTPServerRunner::runner(server.deref()).await
            }
        });


        let task_command = tokio::spawn(async move {
            time::sleep(Duration::from_millis(100)).await;

            let _ = server_c.start(path.clone(), bind_address.clone(), port);
            time::sleep(Duration::from_millis(200)).await;

            //Expected to work; o1=0
            let out_0 = wget::download(url.clone()).await;
            time::sleep(Duration::from_millis(200)).await;

            server_c.stop();
            time::sleep(Duration::from_millis(200)).await;

            //Expected to fail; o2!=0
            let out_1 = wget::download(url.clone()).await;
            time::sleep(Duration::from_millis(200)).await;

            let _ = server_c.start(path.clone(), bind_address.clone(), port);
            time::sleep(Duration::from_millis(200)).await;

            //Expected to work; o3=0
            let out_2 = wget::download(url.clone()).await;
            time::sleep(Duration::from_millis(200)).await;

            server_c.terminate();
            time::sleep(Duration::from_millis(200)).await;

            let out_3 = wget::download(url.clone()).await;
            time::sleep(Duration::from_millis(200)).await;

            (out_0, out_1, out_2, out_3)
        });

        let r = task_command.await.unwrap();
        let _ = task_runner.await;

        return r;
    }
}