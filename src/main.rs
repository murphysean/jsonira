use std::io::Error as IoError;
use std::net::SocketAddr;
use std::path::Path;
use std::str;

use echo::echo;
use http::response::Builder as ResponseBuilder;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Empty, Full};
use hyper::body::{Body, Bytes, Frame};
use hyper::server::conn::{http1, http2};
use hyper::service::service_fn;
use hyper::{header, Method, Request, Response, StatusCode};
use hyper_staticfile::Body as StaticBody;
use hyper_staticfile::Static;
use hyper_util::rt::TokioIo;
use tokio::join;
use tokio::net::TcpListener;
use tracing::{info, span, trace, Instrument as _, Level};
use tracing_subscriber::prelude::*;

mod echo;

#[derive(Clone)]
// An Executor that uses the tokio runtime.
pub struct TokioExecutor;

// Implement the `hyper::rt::Executor` trait for `TokioExecutor` so that it can be used to spawn
// tasks in the hyper runtime.
// An Executor allows us to manage execution of tasks which can help us improve the efficiency and
// scalability of the server.
impl<F> hyper::rt::Executor<F> for TokioExecutor
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn(fut);
    }
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

async fn handle_file_request<B>(
    req: Request<B>,
    static_: Static,
) -> Result<Response<StaticBody>, IoError> {
    if req.uri().path() == "/" {
        let res = ResponseBuilder::new()
            .status(StatusCode::MOVED_PERMANENTLY)
            .header(header::LOCATION, "/index.html")
            .body(StaticBody::Empty)
            .expect("unable to build response");
        Ok(res)
    } else {
        static_.clone().serve(req).await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let console_layer = console_subscriber::spawn();
    let fmt_layer = tracing_subscriber::fmt::layer().pretty();
    tracing_subscriber::registry()
        .with(console_layer)
        .with(fmt_layer)
        .init();
    let http_server_addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let https_server_addr = SocketAddr::from(([0, 0, 0, 0], 8443));

    // the graceful watcher
    let graceful = hyper_util::server::graceful::GracefulShutdown::new();
    let http_server_span = span!(Level::TRACE, "http_server", %http_server_addr);
    let http_server = async move {
        let static_ = Static::new(Path::new("web/"));
        // when this signal completes, start shutdown
        let mut signal = std::pin::pin!(shutdown_signal());
        // We create a TcpListener and bind it to 127.0.0.1:8080
        let http_listener = TcpListener::bind(http_server_addr).await.unwrap();
        trace!("I've bound to the port");

        loop {
            tokio::select! {
                Ok((stream, _addr)) = http_listener.accept() => {
                    trace!("I've accepted a connection");
                    let static_ = static_.clone();
                    tokio::spawn(async move {
                        let io = TokioIo::new(stream);
                        if let Err(e) = http1::Builder::new()
                            .serve_connection(
                                io,
                                service_fn(move |req| handle_file_request(req, static_.clone())),
                            )
                            .await {
                                eprint!("Error serving connection: {:?}", e);
                            };
                        trace!("I've served a request");
                    });
                }
                _ = &mut signal => {
                    eprintln!("graceful shutdown signal received");
                    // stop the accept loop
                    break;
                }
                else => {
                    eprintln!("error/unknown in accept loop");
                }
            }
        }
    };

    let https_server_span = span!(Level::TRACE, "http_server", %https_server_addr);
    let https_server = async move {
        // when this signal completes, start shutdown
        let mut signal = std::pin::pin!(shutdown_signal());
        // We create a TcpListener and bind it to 127.0.0.1:8443
        let https_listener = TcpListener::bind(https_server_addr).await.unwrap();
        // Our server accept loop
        loop {
            tokio::select! {
                Ok((stream, _addr)) = https_listener.accept() => {
                    let io = TokioIo::new(stream);
                    tokio::spawn(async move {
                        if let Err(e) = http2::Builder::new(TokioExecutor)
                        .serve_connection(io, service_fn(echo)).await {
                            eprintln!("Error serving connection: {:?}", e);
                        }
                    });
                },

                _ = &mut signal => {
                    eprintln!("graceful shutdown signal received");
                    // stop the accept loop
                    break;
                }
            }
        }
    };
    //let _ = join!(http_server, https_server);
    let _ = join!(
        http_server.instrument(http_server_span),
        https_server.instrument(https_server_span)
    );

    // Now start the shutdown and wait for them to complete
    // Optional: start a timeout to limit how long to wait.
    tokio::select! {
        _ = graceful.shutdown() => {
            eprintln!("all connections gracefully closed");
        },
        _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
            eprintln!("timed out wait for all connections to close");
        }
    }

    Ok(())
}
