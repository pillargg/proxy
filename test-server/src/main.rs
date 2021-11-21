#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![forbid(unsafe_code)]
//! HTTP server for testing.
//!
//! Runs on port 8080 or a port specified with the `$RELAY_TEST_PORT` environment variable. All
//! successful requests contain the body "success". All failed requests return a 404 with the body
//! "not found".
//!
//! /get accepts GET requests and responds with 200.
//! /post accepts POST requests and responds with 202.

use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tiny_http::{Method, Request, Response, Server};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

fn main() -> Result<()> {
    tracing_subscriber::fmt::try_init()?;
    let server_addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::LOCALHOST),
        env::var("RELAY_TEST_PORT")
            .unwrap_or_else(|_| "8080".to_owned())
            .parse::<u16>()
            .unwrap(),
    );
    let server = Server::http(server_addr)?;
    tracing::info!("server started at {}", server_addr);

    for req in server.incoming_requests() {
        let (url, method) = (req.url().to_owned(), req.method().clone());
        match (url.as_ref(), method) {
            ("/get", Method::Get) | ("/post", Method::Post) => success(req)?,
            _ => not_found(req)?,
        }
    }

    Ok(())
}

#[tracing::instrument(err, skip_all, fields(remote_addr))]
fn success(req: Request) -> Result<()> {
    let remote_addr = req.remote_addr().to_string();
    tracing::Span::current().record::<_, &str>("remote_addr", &remote_addr.as_ref());

    let status = match req.method() {
        Method::Get => 200,
        Method::Post => 202,
        _ => unreachable!(),
    };

    tracing::info!("{} to {}", req.method(), req.url());
    let res = Response::from_string("success").with_status_code(status);
    tracing::info!("responding with {}", status);
    req.respond(res)?;

    Ok(())
}

#[tracing::instrument(err, skip_all, fields(remote_addr))]
fn not_found(req: Request) -> Result<()> {
    let remote_addr = req.remote_addr().to_string();
    tracing::Span::current().record::<_, &str>("remote_addr", &remote_addr.as_ref());

    tracing::warn!("{} to {}", req.method(), req.url());
    let res = Response::from_string("not found").with_status_code(404);
    tracing::warn!("responding with {}", 404);
    req.respond(res)?;

    Ok(())
}
