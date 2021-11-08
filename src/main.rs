#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use std::env;
use std::time::Duration;

use lambda_http::lambda_runtime::{self, Context, Error};
use lambda_http::{handler, IntoResponse, Request, RequestExt, Response};
use reqwest::Client;

use util::{IntoLambdaBody, IntoReqwestBody};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // TODO: move to `.env`.
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("TARGET", "https://rs.fullstory.com"); // Request target with scheme and host.
    env::set_var("TIMEOUT", "10"); // Request timeout, in seconds.

    // Start the Lambda runtime and begin polling for events.
    lambda_runtime::run(handler(entry)).await?;

    // Ok if no error is propogated from the runtime.
    Ok(())
}

/// Takes an HTTP request and a Lambda function execution context, and transforms the request before
/// sending it to the domain specified by the `$TARGET` environment variable. Returns the HTTP
/// response from `$TARGET`.
#[allow(clippy::unused_async)]
async fn entry(req: Request, _: Context) -> Result<impl IntoResponse, Error> {
    // Get environment variables
    let target = env::var("TARGET").expect("Missing environment variable $TARGET");
    let timeout = Duration::from_secs(
        env::var("TIMEOUT")
            .expect("Missing environment variable $TIMEOUT")
            .parse::<u64>()
            .expect("Environment variable $TIMEOUT not a valid u64"),
    );

    // Avoid a drop while borrowing later by getting query params and then converting a reference to
    // it into a Vec.
    let query_params = req.query_string_parameters();
    let query_params: Vec<(&str, &str)> = query_params.iter().collect();
    // Move request into parts and body.
    let (parts, body) = req.into_parts();
    // Parse the url and query parameters for the new request.
    let url = reqwest::Url::parse_with_params(&target, &query_params)?;

    // Creating and send the request, saving the response to return from the Lambda.
    let reqwest_response = Client::builder()
        .build()?
        .request(parts.method, url)
        .version(parts.version)
        .body(body.into_reqwest_body())
        .timeout(timeout)
        .send()
        .await?;

    // Create a new response to return from the Lambda function.
    let mut lambda_response = Response::builder()
        .status(reqwest_response.status())
        .version(reqwest_response.version());

    // Append headers to the response.
    match lambda_response.headers_mut() {
        Some(headers) => {
            todo!()
        }
        _ => (), // FIX: this match arm will only happen if there is a builder error.
    }

    // Convert the reqwest response body to a `lambda_http::Body`.
    let lambda_response =
        lambda_response.body(reqwest_response.bytes().await?.into_lambda_body())?;

    Ok(lambda_response)
}

mod util {
    use bytes::Bytes;

    pub trait IntoReqwestBody {
        fn into_reqwest_body(self) -> reqwest::Body;
    }

    impl IntoReqwestBody for lambda_http::Body {
        /// Converts the `lambda_http::Body` enum into a `reqwest::Body` struct.
        fn into_reqwest_body(self) -> reqwest::Body {
            match self {
                lambda_http::Body::Empty => reqwest::Body::from(""),
                lambda_http::Body::Text(t) => reqwest::Body::from(t),
                lambda_http::Body::Binary(b) => reqwest::Body::from(b),
            }
        }
    }

    pub trait IntoLambdaBody {
        fn into_lambda_body(self) -> lambda_http::Body;
    }

    impl IntoLambdaBody for Bytes {
        /// Zero-clone conversion of `Bytes` into `lambda_http::Body`.
        fn into_lambda_body(self) -> lambda_http::Body {
            if self.is_empty() {
                lambda_http::Body::Empty
            } else {
                // Copy the bytes into a Vec of bytes.
                lambda_http::Body::Binary(self.to_vec())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use lambda_http::http::Method;
    use lambda_http::request::{
        AlbRequestContext, ApiGatewayRequestContext, ApiGatewayV2RequestContext, Http,
    };

    use super::*;

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
