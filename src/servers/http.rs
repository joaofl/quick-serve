use log::{debug, info};

use async_trait::async_trait;
use bytes::Bytes;
use crate::servers::Protocol;
use crate::utils::validation;
use http_body_util::Full;

use hyper_util::rt::TokioIo;
use hyper::{Request, Response};
use hyper::server::conn::http1;
use hyper::service::service_fn;

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use super::Server;
use tokio::net::TcpListener;


async fn receive_request(req: Request<hyper::body::Incoming>, base_path: Arc<PathBuf>) -> Result<Response<Full<Bytes>>, hyper::Error> {

    // Remove the trailing slash from the path to avoid 
    // Path treating it as absolute path and ignoring the base path
    let req_path = req.uri().path().strip_prefix('/').unwrap_or(req.uri().path());

    let file_path = base_path.join(req_path);

    info!("Request path: {}", file_path.display());

    if !file_path.exists() {
        info!("File does not exist: {}", file_path.display());
        return Ok(Response::builder()
            .status(404)
            .body(Full::new(Bytes::from("File not found")))
            .unwrap());
    }

    let file_content = tokio::fs::read(file_path).await.unwrap();
    Ok(Response::new(Full::new(Bytes::from(file_content))))
}


#[async_trait]
pub trait HTTPRunner {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self;
    fn runner(&self);
}

#[async_trait]
impl HTTPRunner for Server {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self {
        let mut s = Server::default();

        validation::validate_path(&path).expect("Invalid path");
        validation::validate_ip_port(&bind_ip, port).expect("Invalid bind IP");

        s.path = Arc::new(path.clone()); // Make a clone of the path and store it in the Server struct
        s.bind_address = IpAddr::from_str(&bind_ip).expect("Invalid IP address");
        s.port = port;

        s.protocol = Protocol::Http;
        HTTPRunner::runner(&s);
        s
    }

    fn runner(&self) {
        let mut receiver = self.sender.subscribe();

        let bind_address = self.bind_address;
        let port = self.port;
        let path = self.path.clone();
        
        tokio::spawn(async move {
            loop {
                debug!("Runner started. Waiting command to connect...");
                let m = receiver.recv().await.unwrap();
                debug!("Message received");

                if m.connect {
                    info!("Connecting...");
                    // Create a SocketAddr from the IpAddr and port

                    let tsk = tokio::spawn(async move {
                        let socket_addr = SocketAddr::new(bind_address, port);
                        let listener = TcpListener::bind(socket_addr).await.unwrap();

                        loop {
                            let (stream, _) = listener.accept().await.unwrap();
                            let io = TokioIo::new(stream);
                            let path_clone = path.clone();

                            info!("Serving...");
                            if let Err(err) = http1::Builder::new()
                                .serve_connection(io, service_fn(move |req| receive_request(req, path_clone.clone())))
                                .await
                            {
                                println!("Error serving connection: {:?}", err);
                            }
                        }
                    });

                    let _ = receiver.recv().await.unwrap();
                    tsk.abort();
                    debug!("HTTP server stopped");
                    break;
                }
            }
        });
    }
}


/////////////////////////////////////////////////////////////////////////////////////
//                                        TESTS                                    //
/////////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use crate::tests::common::tests::*;
    use crate::servers::Protocol;

    #[test]
    fn e2e() {
        let proto = Protocol::Http;
        let port = 8079u16;
        let file_in = "data.bin";
        let file_out = "/tmp/data-out-http.bin";
        let dl_cmd = format!("wget -t2 -T1 {}://127.0.0.1:{}/{} -O {}", proto.to_string(), port, file_in, file_out);

        test_server_e2e(proto, port, dl_cmd, file_in, file_out);
    }
}
