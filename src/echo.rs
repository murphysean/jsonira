
use std::io::Error as IoError;
use std::net::SocketAddr;
use std::path::Path;
use std::str;

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

pub async fn echo(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let span = span!(
        Level::INFO,
        "request",
        method = ?req.method(),
        uri = ?req.uri(),
        headers = ?req.headers()
    );
    async move {
        info!("received request");
        //let mut response = Response::new(Full::new(Bytes::new()));

        match (req.method(), req.uri().path()) {
            // Serve some instructions at /
            (&Method::GET, "/") => {
                const BODY: &str = "Try POSTing data to /echo";
                let response = Response::new(full(BODY));
                info!(body = %(&BODY), "response",);
                Ok(response)
            }

            // Simply echo the body back to the client.
            (&Method::POST, "/echo") => {
                info!(response_kind = %"echo", "response");
                Ok(Response::new(req.into_body().boxed()))
            }

            // Convert to uppercase before sending back to client.
            (&Method::POST, "/echo/uppercase") => {
                // Map this body's frame to a different type
                let frame_stream = req.into_body().map_frame(|frame| {
                    let frame = if let Ok(data) = frame.into_data() {
                        // Convert every byte in every Data frame to uppercase
                        data.iter()
                            .map(|byte| byte.to_ascii_uppercase())
                            .collect::<Bytes>()
                    } else {
                        Bytes::new()
                    };

                    Frame::data(frame)
                });

                info!(response_kind = %"uppercase", "response");
                Ok(Response::new(frame_stream.boxed()))
            }

            // Reverse the entire body before sending back to the client.
            (&Method::POST, "/echo/reversed") => {
                // Protect our server from massive bodies.
                let upper = req.body().size_hint().upper().unwrap_or(u64::MAX);
                if upper > 1024 * 64 {
                    let mut resp = Response::new(full("Body too big"));
                    *resp.status_mut() = hyper::StatusCode::PAYLOAD_TOO_LARGE;
                    return Ok(resp);
                }

                // Await the whole body to be collected into a single `Bytes`...
                let whole_body = req.collect().await?.to_bytes();

                // Iterate the whole body in reverse order and collect into a new Vec.
                let reversed_body = whole_body.iter().rev().cloned().collect::<Vec<u8>>();

                Ok(Response::new(full(reversed_body)))
            }

            // The 404 Not Found route...
            _ => {
                let mut not_found = Response::new(empty());
                *not_found.status_mut() = StatusCode::NOT_FOUND;
                info!(
                    body = ?(),
                    status = ?StatusCode::NOT_FOUND,
                    "response",
                );
                Ok(not_found)
            }
        }
    }
    .instrument(span)
    .await
}

// We create some utility functions to make Empty and Full bodies
// fit our broadened Response body type.
fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}