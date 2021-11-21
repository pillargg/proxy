#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![forbid(unsafe_code)]
//! HTTP server for testing.
//!
//! Runs on port 8888 or a port specified with the `$RELAY_TEST_PORT` environment variable. All
//! successful requests return a 200 with the body "success". All failed requests return a 404 with
//! the body "not found".
//!
//! |   Path   | Request  | Status |
//! |----------|----------|--------|
//! | /get     | GET      | 200    |
//! | /head    | HEAD     | 200    |
//! | /post    | POST     | 200    |
//! | /put     | PUT      | 200    |
//! | /delete  | DELETE   | 200    |
//! | /connect | CONNNECT | 200    |
//! | /options | OPTIONS  | 200    |
//! | /trace   | TRACE    | 200    |
//! | /patch   | PATCH    | 200    |

use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tiny_http::{Method, Request, Response, Server};

/// Error type for test HTTP server `Result`s.
type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
/// Result type for test HTTP server functions.
type Result = std::result::Result<(), Error>;

fn main() -> Result {
    // Initialize the tracing subscriber.
    tracing_subscriber::fmt::try_init()?;

    // Create a new socket address to listen on and start a server at that address.
    let server_addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::LOCALHOST),
        match env::var("RELAY_TEST_PORT") {
            Ok(p) => p.parse::<u16>().unwrap(),
            Err(_) => 8888,
        },
    );
    let server = Server::http(server_addr)?;
    tracing::info!("server started at {}", server_addr);

    // Loop through requests and direct them to the right place.
    for req in server.incoming_requests() {
        let (url, method) = (req.url(), req.method().clone());
        match (url, method) {
            ("/get", Method::Get)
            | ("/head", Method::Head)
            | ("/post", Method::Post)
            | ("/put", Method::Put)
            | ("/delete", Method::Delete)
            | ("/connect", Method::Connect)
            | ("/options", Method::Options)
            | ("/trace", Method::Trace)
            | ("/patch", Method::Patch) => {
                success(req)?;
            }
            _ => not_found(req)?,
        }
    }

    Ok(())
}

/// Process a request and respond with a successful status code and message.
#[tracing::instrument(err, skip_all, fields(remote_addr))]
fn success(req: Request) -> Result {
    // Record the request's source address as a span field.
    let remote_addr = &*req.remote_addr().to_string();
    tracing::Span::current().record("remote_addr", &remote_addr);
    // Trace event with the request info.
    tracing::info!("{} to {}", req.method(), req.url());

    // Create and send the response.
    let res = Response::from_string("success").with_status_code(200);
    tracing::info!("responding with {}", 200);
    req.respond(res)?;

    Ok(())
}

/// Process a request and respond with a 404 status code and "not found" message.
#[tracing::instrument(err, skip_all, fields(remote_addr))]
fn not_found(req: Request) -> Result {
    // Record the request's source address as a span field.
    let remote_addr = &*req.remote_addr().to_string();
    tracing::Span::current().record("remote_addr", &remote_addr);
    // Trace event with the request info.
    tracing::warn!("{} to {}", req.method(), req.url());

    // Create and send the response.
    let res = Response::from_string("not found").with_status_code(404);
    tracing::warn!("responding with {}", 404);
    req.respond(res)?;

    Ok(())
}
