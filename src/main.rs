#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use lambda_http::lambda_runtime::{self, Context, Error};
use lambda_http::{handler, IntoResponse, Request, RequestExt, Response};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_BACKTRACE", "1"); // TODO: move to .env
    std::env::set_var("TARGET", "rs.fullstory.com"); // TODO: move to .env

    lambda_runtime::run(handler(entry)).await?;
    Ok(())
}

#[allow(clippy::unused_async)]
async fn entry(req: Request, _: Context) -> Result<impl IntoResponse, Error> {
    // Cache the query parameters.
    let query_params = req.query_string_parameters();
    // Get the source address.
    let source_addr = get_source_addr(&req);
    // Separate the `Request` into head and body.
    let (mut parts, body) = req.into_parts();

    // Convert `lambda_http::Request` into a `hyper::Request`.
    let body = match body {
        Body::Empty => hyper::Body::empty(),
        Body::Text(t) => hyper::Body::from(t.into_bytes()),
        Body::Binary(b) => hyper::Body::from(b),
    };
    let mut uri = format!(
        "https://{}{}",
        std::env::var("TARGET").expect("Missing environment variable $TARGET"),
        parts.uri.path()
    );

    // URL encode the query parameters again and place them back into the URI because
    // `lambda_runtime` automatically parses query params AND removes those query parameters from
    // the original URI.
    if !query_params.is_empty() {
        append_querystring_from_map(&mut uri, query_params.iter());
    }

    // Convert the `String` into a `hyper::Uri`.
    parts.uri = match uri.parse::<hyper::Uri>() {
        Ok(uri) => uri,
        Err(e) => panic!("Failed to build uri: {:?}", e),
    };

    // Rebuild the request as a `hyper::Request` using its modified parts.
    let req = hyper::Request::from_parts(parts, body);

    // TODO: relay request
    // let response = lambda_http::http::request::Builder::new()
    //     .method("POST")
    //     .body(())
    //     .unwrap();

    // Convert the `hyper::Request` into a `lambda_http::Request`.
    // let (parts, body) = response.into_parts();
    let body_bytes = hyper::body::to_bytes(body).await?;
    let body = String::from_utf8(body_bytes.to_vec()).unwrap();
    Ok(Response::from_parts(parts, Body::from(body)))
    // Ok(match query_params.get("first_name") {
    //     Some(first_name) => format!("Hello, {}!", first_name).into_response(),
    //     _ => Response::builder()
    //         .status(400)
    //         .body("Empty first name".into())
    //         .expect("Failed to render response"),
    // })
}

fn get_source_addr(req: &Request) -> SocketAddr {
    let source_ip: String = match req.request_context() {
        RequestContext::ApiGateway(ApiGatewayRequestContext { identity, .. }) => identity.source_ip,
        RequestContext::ApiGatewayV2(ApiGatewayV2RequestContext {
            http: Http { source_ip, .. },
            ..
        }) => source_ip,
        RequestContext::Alb(_) => Ipv4Addr::UNSPECIFIED.to_string(),
    };

    SocketAddr::new(
        source_ip.parse().unwrap(),
        std::env::var("PORT")
            .expect("Missing environment variable $PORT")
            .parse::<u16>()
            .expect("$PORT environment variable not a valid port"),
    )
}

fn append_querystring_from_map<'a, I>(uri: &mut String, from_query_params: I)
where
    I: Iterator<Item = (&'a str, &'a str)>,
{
    uri.push('?');
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for (key, value) in from_query_params.into_iter() {
        serializer.append_pair(key, value);
    }
    uri.push_str(serializer.finish().as_str())
}
