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

use crate::util::{GetRequestId, GetSourceIp, IntoLambdaBody, IntoReqwestBody};

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    // Install a global tracing subscriber that listens for events and filters based on `$RUST_LOG`.
    tracing_subscriber::fmt::try_init()?;

    // TODO: use Lambda env vars
    env::set_var("RUST_LOG", "TRACE");
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
    err,      // Emit any errors from the function.
    skip_all, // Skip all fields except the ones below.
    fields(
        source.id = ?req.id(),
        source.addr = ?req.source_ip(),
    ),
)]
async fn entry(req: Request, _: Context) -> Result<impl IntoResponse, lambda_runtime::Error> {
    tracing::trace!(source.method = %req.method());
    tracing::trace!(source.version = ?req.version());

    // Get environment variables
    let target = env::var("RELAY_TARGET").expect("Missing $RELAY_TARGET environment variable");
    tracing::trace!(%target);
    let timeout = env::var("RELAY_TIMEOUT")
        .expect("Missing $RELAY_TIMEOUT environment variable")
        .parse::<u64>()
        .expect("$RELAY_TIMEOUT must be a valid u64");
    tracing::trace!(timeout);
    let timeout = Duration::from_secs(timeout);

    // Move request into parts and body.
    let (parts, body) = req.into_parts();
    tracing::trace!(?body);

    // Join the url, path, and query parameters for the new request.
    let mut url = Url::parse(&target)?;
    url.set_path(parts.uri.path());
    url.set_query(parts.uri.query());
    tracing::info!(parsed_url = %url);

    // Build the client.
    let mut client = Client::builder().use_rustls_tls();
    if parts.version == Version::HTTP_2 {
        client = client.http2_prior_knowledge();
    }
    tracing::trace!(client_builder = ?client);
    let client = client.build()?;
    tracing::trace!(?client);

    // Build and send the request.
    let req = client
        .request(parts.method, url)
        .headers(parts.headers)
        .body(body.into_reqwest_body())
        .version(parts.version)
        .timeout(timeout);
    tracing::info!(request = ?req);

    let reqwest_res = req.send().await?;

    // Create a new response to return from the Lambda function.
    let mut lambda_res = Response::builder()
        .status(reqwest_res.status())
        .version(reqwest_res.version());

    // Add the response headers. Unwrap is safe because we  builder errors.
    let headers = lambda_res.headers_mut().unwrap();
    // Clone: need ownership of the headers but `http::HeaderMap` is not `Copy`.
    *headers = reqwest_res.headers().clone();

    // Add the response body.
    let lambda_res = lambda_res.body(reqwest_res.bytes().await?.into_lambda_body())?;

    Ok(lambda_res)
}
