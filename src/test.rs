#![cfg(test)]

use std::env;

use lambda_http::http::{Method, StatusCode, Version};
use lambda_http::request::RequestContext;
use lambda_http::{Body, Context, IntoResponse, Request};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::entry;
use crate::util::test::MockRequestContext;

// HTTP/1.1 POST payload.
#[tokio::test]
async fn test_post_http11() {
    // Start the mock server and get its URL.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(202).set_body_string("success"))
        .mount(&server)
        .await;
    let target = server.address().to_string();

    env::set_var("RUST_LOG", "TRACE");
    env::set_var("RELAY_TARGET", &format!("{}{}", "http://", &target));
    env::set_var("RELAY_TIMEOUT", "3");

    let mut request = Request::new(Body::from("test"));

    *request.uri_mut() = "https://example.com/test".parse().unwrap();
    *request.method_mut() = Method::POST;
    *request.version_mut() = Version::HTTP_11;
    request
        .extensions_mut()
        .insert(RequestContext::mock(Method::POST));

    // Test the `entry` function.
    let response = entry(request, Context::default())
        .await
        .unwrap()
        .into_response();

    assert_eq!(StatusCode::ACCEPTED, response.status());
    assert_eq!(response.into_body(), Body::Binary(b"success".to_vec()));
}

// HTTP/2 POST payload.
#[tokio::test]
async fn test_post_http2() {
    // Start the mock server and get its URL.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(202).set_body_string("success"))
        .mount(&server)
        .await;
    let target = server.address().to_string();

    env::set_var("RUST_LOG", "TRACE");
    env::set_var("RELAY_TARGET", &format!("{}{}", "http://", &target));
    env::set_var("RELAY_TIMEOUT", "3");

    let mut request = Request::new(Body::from("test"));

    *request.uri_mut() = "https://example.com/test".parse().unwrap();
    *request.method_mut() = Method::POST;
    *request.version_mut() = Version::HTTP_2;
    request
        .extensions_mut()
        .insert(RequestContext::mock(Method::POST));

    // Test the `entry` function.
    let response = entry(request, Context::default())
        .await
        .unwrap()
        .into_response();

    assert_eq!(StatusCode::ACCEPTED, response.status());
    assert_eq!(response.into_body(), Body::Binary(b"success".to_vec()));
}

// TODO: wiremock needs https support
// #[tokio::test]
// async fn test_post_https() {
//     let server = MockServer::start().await;
//     Mock::given(method("POST"))
//         .and(path("/test"))
//         .respond_with(ResponseTemplate::new(202).set_body_string("success"))
//         .mount(&server)
//         .await;
//     let target = server.address().to_string();

//     env::set_var("RUST_LOG", "TRACE");
//     env::set_var("RELAY_TARGET", &format!("{}{}", "https://", &target));
//     env::set_var("RELAY_TIMEOUT", "3");

//     let mut request = Request::new(Body::from("test"));

//     *request.uri_mut() = "https://example.com/test".parse().unwrap();
//     *request.method_mut() = Method::POST;
//     *request.version_mut() = Version::HTTP_2;
//     request
//         .extensions_mut()
//         .insert(RequestContext::mock(Method::POST));

//     // Test the function.
//     let response = entry(request, Context::default())
//         .await
//         .unwrap()
//         .into_response();

//     assert_eq!(StatusCode::ACCEPTED, response.status());
//     assert_eq!(response.into_body(), Body::Binary(b"success".to_vec()));
// }
