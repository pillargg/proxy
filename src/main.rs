#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
// https://github.com/awslabs/aws-lambda-rust-runtime/commit/ba9040ceec6dd1cd1273cb2d0359f0f504f5417b
#![allow(clippy::multiple_crate_versions)]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

#[cfg(test)]
mod test;
mod util;

use std::env;
use std::time::Duration;

use lambda_http::http::Version;
use lambda_http::lambda_runtime::{self, Context};
use lambda_http::{IntoResponse, Request, Response};
use reqwest::{Client, Url};

use util::{IntoLambdaBody as _, IntoReqwestBody as _, SourceAddr as _};

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    tracing_subscriber::fmt::try_init()?;

    // TODO: use context.env_config
    env::set_var("RELAY_TARGET", "https://rs.fullstory.com");
    env::set_var("RELAY_TIMEOUT", "10");

    // Start the Lambda runtime and begin polling for events.
    lambda_runtime::run(lambda_http::handler(entry)).await?;

    Ok(())
}

/// Takes an HTTP request and a Lambda function execution context, and transforms the request before
/// sending it to the domain specified by the `$RELAY_TARGET` environment variable. Returns the HTTP
/// response.
#[tracing::instrument(
    level = "info",
    err,
    skip_all,
    fields(
        // source_addr = req.source_addr(),
        uri = %req.uri(),
        method = %req.method(),
        // version = req.version(),
    ),
)]
async fn entry(req: Request, _: Context) -> Result<impl IntoResponse, lambda_runtime::Error> {
    // Get environment variables
    let target = env::var("RELAY_TARGET").expect("Missing $RELAY_TARGET environment variable");
    let timeout = Duration::from_secs(
        env::var("RELAY_TIMEOUT")
            .expect("Missing $RELAY_TIMEOUT environment variable")
            .parse::<u64>()
            .expect("$RELAY_TIMEOUT must be a valid u64"),
    );

    // Move request into parts and body.
    let (parts, body) = req.into_parts();

    // Join the url, path, and query parameters for the new request.
    let mut url = Url::parse(&target).unwrap();
    url.set_path(parts.uri.path());
    url.set_query(parts.uri.query());

    // Build the client.
    let mut reqwest_client = Client::builder().use_rustls_tls();
    if parts.version == Version::HTTP_2 {
        reqwest_client = reqwest_client.http2_prior_knowledge();
    }
    let reqwest_client = reqwest_client.build().unwrap();

    // Build and send the request.
    let reqwest_response = reqwest_client
        .request(parts.method, url)
        .headers(parts.headers)
        .body(body.into_reqwest_body())
        .version(parts.version)
        .timeout(timeout)
        .send()
        .await
        .unwrap();

    // Create a new response to return from the Lambda function.
    let mut lambda_response = Response::builder()
        .status(reqwest_response.status())
        .version(reqwest_response.version());

    // Add the response headers.
    let headers = lambda_response.headers_mut().unwrap();
    // Clone: need ownership of the headers but `http::HeaderMap` is not `Copy`.
    *headers = reqwest_response.headers().clone();

    // Add the response body.
    let lambda_response = lambda_response
        .body(reqwest_response.bytes().await.unwrap().into_lambda_body())
        .unwrap();

    Ok(lambda_response)
}
