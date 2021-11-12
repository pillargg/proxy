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
    use std::collections::HashMap;

    use lambda_http::http::{Method, StatusCode};
    use lambda_http::request::{ApiGatewayV2RequestContext, Http, RequestContext};
    use lambda_http::{Body, Request};
    use mockito::mock;

    use super::*;

    #[tokio::test]
    async fn test_entry_201() {
        let mock = mock("POST", "/test")
            .with_status(201)
            .with_header("content-type", "text/plain")
            .with_header("x-api-key", "1234")
            .with_body("test")
            .create();

        let url = &mockito::server_url();

        env::set_var("RELAY_TARGET", url);
        env::set_var("RELAY_TIMEOUT", "10");

        let mut request = Request::new(Body::from("test"));
        *request.uri_mut() = "https://test.com/test".parse().unwrap();

        request
            .extensions_mut()
            .insert::<RequestContext>(RequestContext::ApiGatewayV2(ApiGatewayV2RequestContext {
                account_id: String::new(),
                api_id: String::new(),
                authorizer: HashMap::new(),
                domain_name: String::new(),
                domain_prefix: String::new(),
                http: Http {
                    method: Method::POST,
                    path: "/test".to_owned(),
                    protocol: String::new(),
                    source_ip: "127.0.0.1".to_owned(),
                    user_agent: String::new(),
                },
                request_id: String::new(),
                route_key: String::new(),
                stage: String::new(),
                time: String::new(),
                time_epoch: 0,
            }));

        {
            let response = entry(request, Context::default())
                .await
                .unwrap()
                .into_response();
        }
        mock.assert();

        // assert_eq!(StatusCode::OK, response.status());
        // assert_eq!(response.into_body(), lambda_http::Body::from("test"));
    }
}
