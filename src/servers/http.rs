use log::{debug, info, error};

use bytes::Bytes;
use crate::servers::Protocol;
use crate::utils::validation;
use http_body_util::Full;

use hyper_util::rt::TokioIo;
use hyper::{Request, Response, StatusCode};
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

    // Use the new validation function for security checks
    let file_path = match crate::utils::validation::validate_file_path(&base_path, req_path) {
        Ok(path) => path,
        Err(e) => {
            error!("Path validation failed for '{}': {}", req_path, e);
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from("Invalid path")))
                .unwrap());
        }
    };

    info!("Request path: {}", file_path.display());

    if !file_path.exists() {
        info!("File does not exist: {}", file_path.display());
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from("File not found")))
            .unwrap());
    }

    if !file_path.is_file() {
        info!("Path is not a file: {}", file_path.display());
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Full::new(Bytes::from("Path is not a file")))
            .unwrap());
    }

    match tokio::fs::read(&file_path).await {
        Ok(file_content) => {
            info!("Successfully served file: {} ({} bytes)", file_path.display(), file_content.len());
            Ok(Response::new(Full::new(Bytes::from(file_content))))
        }
        Err(e) => {
            error!("Failed to read file {}: {}", file_path.display(), e);
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from("Internal server error")))
                .unwrap())
        }
    }
}


pub trait HTTPRunner {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self;
    fn runner(&self);
}

impl HTTPRunner for Server {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self {
        let mut s = Server::default();

        // Validate inputs with proper error handling
        if let Err(e) = validation::validate_path(&path) {
            error!("Invalid path '{}': {}", path.display(), e);
            panic!("Invalid path: {}", e);
        }
        
        if let Err(e) = validation::validate_ip_port(&bind_ip, port) {
            error!("Invalid bind IP '{}:{}': {}", bind_ip, port, e);
            panic!("Invalid bind IP: {}", e);
        }

        s.path = Arc::new(path.clone()); // Make a clone of the path and store it in the Server struct
        s.bind_address = match IpAddr::from_str(&bind_ip) {
            Ok(addr) => addr,
            Err(e) => {
                error!("Failed to parse IP address '{}': {}", bind_ip, e);
                panic!("Invalid IP address: {}", e);
            }
        };
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
                debug!("HTTP runner started. Waiting command to connect...");
                
                let m = match receiver.recv().await {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("Failed to receive message in HTTP runner: {}", e);
                        break;
                    }
                };
                debug!("Message received");

                if m.connect {
                    info!("Starting HTTP server on {}:{}", bind_address, port);
                    
                    let tsk = tokio::spawn(async move {
                        let socket_addr = SocketAddr::new(bind_address, port);
                        
                        let listener = match TcpListener::bind(socket_addr).await {
                            Ok(listener) => {
                                info!("HTTP server listening on {}", socket_addr);
                                listener
                            }
                            Err(e) => {
                                error!("Failed to bind HTTP server to {}: {}", socket_addr, e);
                                return;
                            }
                        };

                        loop {
                            match listener.accept().await {
                                Ok((stream, addr)) => {
                                    debug!("New HTTP connection from {}", addr);
                                    let io = TokioIo::new(stream);
                                    let path_clone = path.clone();

                                    tokio::spawn(async move {
                                        if let Err(err) = http1::Builder::new()
                                            .serve_connection(io, service_fn(move |req| receive_request(req, path_clone.clone())))
                                            .await
                                        {
                                            error!("Error serving HTTP connection from {}: {:?}", addr, err);
                                        }
                                    });
                                }
                                Err(e) => {
                                    error!("Failed to accept HTTP connection: {}", e);
                                    // Continue accepting other connections
                                }
                            }
                        }
                    });

                    // Wait for stop command
                    match receiver.recv().await {
                        Ok(_) => {
                            info!("Stop command received, shutting down HTTP server");
                            tsk.abort();
                            debug!("HTTP server stopped");
                            break;
                        }
                        Err(e) => {
                            error!("Failed to receive stop command: {}", e);
                            tsk.abort();
                            break;
                        }
                    }
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
