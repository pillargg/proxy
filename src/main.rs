#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![forbid(unsafe_code)]
// https://github.com/awslabs/aws-lambda-rust-runtime/commit/ba9040ceec6dd1cd1273cb2d0359f0f504f5417b
#![allow(clippy::multiple_crate_versions)]
#![doc = include_str!("../README.md")]

#[cfg(test)]
mod test;
mod util;

use std::env;
use std::time::Duration;

use lambda_http::lambda_runtime::{self, Context};
use lambda_http::{IntoResponse, Request, Response};
use reqwest::Client;

use crate::util::{GetContextInfo, GetRequestContext, IntoLambdaBody, IntoReqwestBody};

lazy_static::lazy_static! {
    static ref RELAY_TARGET: String = env::var("RELAY_TARGET").unwrap();
    static ref CLIENT: Client = Client::builder()
        .use_rustls_tls()
        .http2_prior_knowledge()
        .timeout(Duration::from_secs(
            env::var("RELAY_TIMEOUT")
                .unwrap()
                .parse::<u64>()
                .unwrap()
        ))
        .build()
        .unwrap();
}

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    // Install a global tracing subscriber that listens for events and filters based on `$RUST_LOG`.
    tracing_subscriber::fmt::try_init()?;

    // Start the Lambda runtime and begin polling for events.
    lambda_runtime::run(lambda_http::handler(entry)).await
}

/// Takes an HTTP request and a Lambda function execution context, and transforms the request before
/// sending it to the domain specified by the `$RELAY_TARGET`. Returns the HTTP response.
#[tracing::instrument(
    err,      // Emit any errors from the function.
    skip_all, // Skip all fields except the ones below.
    fields(req.id, req.ip),
)]
async fn entry(req: Request, _: Context) -> Result<impl IntoResponse, lambda_runtime::Error> {
    let ctx = req.context()?;
    tracing::Span::current().record("req.id", &ctx.request_id());
    tracing::Span::current().record("req.ip", &ctx.source_ip());
    tracing::trace!(req.method = %req.method(), req.version = ?req.version());

    // Move request into parts and body.
    let (parts, body) = req.into_parts();
    tracing::trace!(req.parts = ?parts, req.body = ?body);

    // Build and send the request.
    tracing::info!("{} to target", parts.method);
    let req = CLIENT
        .request::<&str>(parts.method, &RELAY_TARGET)
        .headers(parts.headers)
        .version(parts.version)
        .body(body.into_reqwest_body());

    let reqwest_res = req.send().await?;
    tracing::info!("{} from target", reqwest_res.status());

    // Create a new response to return from the Lambda function.
    let mut lambda_res = Response::builder()
        .status(reqwest_res.status())
        .version(reqwest_res.version());

    // Unwrap: at this point a builder error should not be possible.
    let headers = lambda_res.headers_mut().unwrap();
    // Clone: move the `http::HeaderMap` into headers.
    *headers = reqwest_res.headers().clone();
    tracing::trace!(res.parts = ?lambda_res);

    // Add the response body.
    let body = reqwest_res.bytes().await?.into_lambda_body();
    tracing::trace!(res.body = ?body);
    let lambda_res = lambda_res.body(body)?;

    Ok(lambda_res)
}
