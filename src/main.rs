#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use lambda_http::handler;
use lambda_http::{IntoResponse, Request, RequestExt, Response};
use lambda_runtime::{Context, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(handler(func)).await?;

    Ok(())
}

#[allow(clippy::unused_async)]
async fn func(event: Request, _: Context) -> Result<impl IntoResponse, Error> {
    Ok(match event.query_string_parameters().get("first_name") {
        Some(first_name) => format!("Hello, {}!", first_name).into_response(),
        _ => Response::builder()
            .status(400)
            .body("Empty first name".into())
            .expect("Failed to render response"),
    })
}
