//! Utility traits.

use lambda_http::request::{
    ApiGatewayRequestContext, ApiGatewayV2RequestContext, Http, Identity, RequestContext,
};
use lambda_http::RequestExt;

/// Used to obtain the source IP address of a `lambda_http::RequestContext` if
/// it exists.
pub trait SourceAddr {
    /// Retrieves the source IP address, if available.
    fn source_addr(&self) -> Option<String>;
}

impl SourceAddr for lambda_http::Request {
    fn source_addr(&self) -> Option<String> {
        match self.request_context() {
            RequestContext::ApiGatewayV2(ApiGatewayV2RequestContext {
                http: Http { source_ip, .. },
                ..
            })
            | RequestContext::ApiGateway(ApiGatewayRequestContext {
                identity: Identity { source_ip, .. },
                ..
            }) => Some(source_ip),
            RequestContext::Alb(_) => None,
        }
    }
}

/// A value-to-value conversion into a `reqwest::Body`, consuming the input.
pub trait IntoReqwestBody {
    /// Performs the conversion.
    fn into_reqwest_body(self) -> reqwest::Body;
}

impl IntoReqwestBody for lambda_http::Body {
    fn into_reqwest_body(self) -> reqwest::Body {
        match self {
            Self::Empty => reqwest::Body::from(""),
            Self::Text(t) => reqwest::Body::from(t),
            Self::Binary(b) => reqwest::Body::from(b),
        }
    }
}

/// A value-to-value conversion into a `lambda_http::Body`, consuming the input.
pub trait IntoLambdaBody {
    /// Performs the conversion.
    fn into_lambda_body(self) -> lambda_http::Body;
}

impl IntoLambdaBody for bytes::Bytes {
    fn into_lambda_body(self) -> lambda_http::Body {
        if self.is_empty() {
            lambda_http::Body::Empty
        } else {
            lambda_http::Body::Binary(self.to_vec())
        }
    }
}
