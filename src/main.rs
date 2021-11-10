#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use std::env;
use std::time::Duration;

use lambda_http::lambda_runtime::{self, Context};
use lambda_http::{handler, IntoResponse, Request, RequestExt, Response};
use reqwest::{Client, Url};

use util::{IntoLambdaBody as _, IntoReqwestBody as _};

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    // TODO: move to `.env`.
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RELAY_TARGET", "https://rs.fullstory.com"); // Request target with scheme and host.
    env::set_var("RELAY_TIMEOUT", "10"); // Request timeout, in seconds.

    // Start the Lambda runtime and begin polling for events.
    lambda_runtime::run(handler(entry)).await?;

    // Ok if no error is propagated from the runtime.
    Ok(())
}

/// Takes an HTTP request and a Lambda function execution context, and
/// transforms the request before sending it to the domain specified by the
/// `$RELAY_TARGET` environment variable. Returns the HTTP response.
async fn entry(req: Request, _: Context) -> Result<impl IntoResponse, lambda_runtime::Error> {
    // Get environment variables
    let target = env::var("RELAY_TARGET").expect("Missing environment variable $RELAY_TARGET");
    let timeout = Duration::from_secs(
        env::var("RELAY_TIMEOUT")
            .expect("Missing environment variable $RELAY_TIMEOUT")
            .parse::<u64>()
            .expect("Environment variable $RELAY_TIMEOUT not a valid u64"),
    );

    // Parse the url and query parameters for the new request.
    let query_params = req.query_string_parameters();
    let url = Url::parse_with_params(&target, query_params.iter())?;

    // Move request into parts and body.
    let (parts, body) = req.into_parts();

    // Creating and send the request, saving the response.
    let reqwest_response = Client::builder()
        .build()?
        .request(parts.method, url)
        .headers(parts.headers)
        .body(body.into_reqwest_body())
        .version(parts.version)
        .timeout(timeout)
        .send()
        .await?;

    // Create a new response to return from the Lambda function.
    let mut lambda_response = Response::builder()
        .status(reqwest_response.status())
        .version(reqwest_response.version());

    // Get a mutable ref to the response headers.
    let headers = lambda_response.headers_mut().unwrap();
    // Replace the Lambda response's headers with the reqwest response's.
    // Clone: need ownership of the headers but `http::HeaderMap` is not `Copy`.
    *headers = reqwest_response.headers().clone();

    // Convert the reqwest response body to a `lambda_http::Body`.
    let lambda_response =
        lambda_response.body(reqwest_response.bytes().await?.into_lambda_body())?;

    Ok(lambda_response)
}

mod util {
    pub trait IntoReqwestBody {
        fn into_reqwest_body(self) -> reqwest::Body;
    }

    impl IntoReqwestBody for lambda_http::Body {
        /// Convert the `lambda_http::Body` into a `reqwest::Body`.
        fn into_reqwest_body(self) -> reqwest::Body {
            match self {
                Self::Empty => reqwest::Body::from(""),
                Self::Text(t) => reqwest::Body::from(t),
                Self::Binary(b) => reqwest::Body::from(b),
            }
        }
    }

    pub trait IntoLambdaBody {
        fn into_lambda_body(self) -> lambda_http::Body;
    }

    impl IntoLambdaBody for bytes::Bytes {
        /// Convert the `Bytes` into a `lambda_http::Body`.
        fn into_lambda_body(self) -> lambda_http::Body {
            if self.is_empty() {
                lambda_http::Body::Empty
            } else {
                lambda_http::Body::Binary(self.to_vec())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // use lambda_http::http::Method;
    // use lambda_http::request::{
    //     AlbRequestContext, ApiGatewayRequestContext, ApiGatewayV2RequestContext, Http,
    // };

    // use super::*;

    // #[tokio::test]
    // async fn return_200_and_response() {
    //     let mut request = lambda_http::Request::new(lambda_http::Body::Empty);
    //     *request.uri_mut() = "/".parse().unwrap();
    //     request
    //         .extensions_mut()
    //         .insert::<RequestContext>(RequestContext::ApiGatewayV2(ApiGatewayV2RequestContext {
    //             route_key: None,
    //             account_id: None,
    //             stage: None,
    //             request_id: None,
    //             authorizer: None,
    //             apiid: None,
    //             domain_name: None,
    //             domain_prefix: None,
    //             time: None,
    //             time_epoch: 0,
    //             http: Http {
    //                 method: Method::GET,
    //                 path: None,
    //                 protocol: None,
    //                 source_ip: Some("127.0.0.1".to_string()),
    //                 user_agent: None,
    //             },
    //         }));
    //     let response = entrypoint(request, Context::default())
    //         .await
    //         .unwrap()
    //         .into_response();
    //     assert_eq!(StatusCode::OK, response.status());
    //     assert!(!response.body().is_empty())
    // }
}
