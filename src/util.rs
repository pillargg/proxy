//! Utility traits.

use lambda_http::lambda_runtime;
use lambda_http::request::{
    ApiGatewayRequestContext, ApiGatewayV2RequestContext, Http, Identity, RequestContext,
};

/// Get the request context from a `lambda_http::Request`'s extensions.
pub trait GetRequestContext<'req> {
    /// Get the request context if it exists.
    fn context(&'req self) -> Result<&'req RequestContext, lambda_runtime::Error>;
}

impl<'req> GetRequestContext<'req> for lambda_http::Request {
    fn context(&'req self) -> Result<&'req RequestContext, lambda_runtime::Error> {
        Ok(self
            .extensions()
            .get::<RequestContext>()
            .ok_or("request did not contain a request context")?)
    }
}

/// Extract information from a `RequestContext`.
pub trait GetContextInfo<'req> {
    /// Get the request ID.
    fn request_id(&'req self) -> &'req str;
    /// Get the source IP address.
    fn source_ip(&'req self) -> &'req str;
}

impl<'req> GetContextInfo<'req> for RequestContext {
    fn request_id(&'req self) -> &'req str {
        match self {
            RequestContext::ApiGatewayV2(ApiGatewayV2RequestContext { request_id, .. })
            | RequestContext::ApiGateway(ApiGatewayRequestContext { request_id, .. }) => request_id,
            RequestContext::Alb(_) => "",
        }
    }

    fn source_ip(&'req self) -> &'req str {
        match self {
            RequestContext::ApiGatewayV2(ApiGatewayV2RequestContext {
                http: Http { source_ip, .. },
                ..
            })
            | RequestContext::ApiGateway(ApiGatewayRequestContext {
                identity: Identity { source_ip, .. },
                ..
            }) => source_ip,
            RequestContext::Alb(_) => "",
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

/// Utility traits for tests.
#[cfg(test)]
pub mod test {
    use std::collections::HashMap;

    use lambda_http::http::Method;
    use lambda_http::request::{ApiGatewayV2RequestContext, Http, RequestContext};

    /// A mock API Gateway v2 request context to be added to an HTTP request's extensions.
    pub trait MockRequestContext {
        /// Creates a mock request context.
        fn mock(m: Method) -> Self
        where
            Self: Sized;
    }

    impl MockRequestContext for RequestContext {
        fn mock(m: Method) -> Self {
            Self::ApiGatewayV2(ApiGatewayV2RequestContext {
                account_id: "".to_owned(),
                api_id: "".to_owned(),
                authorizer: HashMap::default(),
                domain_name: "".to_owned(),
                domain_prefix: "".to_owned(),
                http: Http {
                    method: m,
                    path: "".to_owned(),
                    protocol: "".to_owned(),
                    source_ip: "127.0.0.1".to_owned(),
                    user_agent: "".to_owned(),
                },
                request_id: "".to_owned(),
                route_key: "".to_owned(),
                stage: "".to_owned(),
                time: "".to_owned(),
                time_epoch: 0,
            })
        }
    }
}
