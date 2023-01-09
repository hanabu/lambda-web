// SPDX-License-Identifier: MIT
//!
//! Run hyper based web framework on AWS Lambda
//!
use crate::request::LambdaHttpEvent;
use core::convert::TryFrom;
use core::future::Future;
use lambda_runtime::{Error as LambdaError, LambdaEvent, Service as LambdaService};
use std::pin::Pin;

type HyperRequest = hyper::Request<hyper::Body>;
type HyperResponse<B> = hyper::Response<B>;

/// Run hyper based web framework on AWS Lambda
///
/// axum 0.3 example:
///
/// ```no_run
/// use axum::{routing::get, Router};
/// use lambda_web::{is_running_on_lambda, run_hyper_on_lambda, LambdaError};
/// use std::net::SocketAddr;
///
/// // basic handler that responds with a static string
/// async fn root() -> &'static str {
///     "Hello, World!"
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), LambdaError> {
///     // build our application with a route
///     let app = Router::new()
///         // `GET /` goes to `root`
///         .route("/", get(root));
///
///     if is_running_on_lambda() {
///         // Run app on AWS Lambda
///         run_hyper_on_lambda(app).await?;
///     } else {
///         // Run app on local server
///         let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
///         axum::Server::bind(&addr).serve(app.into_make_service()).await?;
///     }
///     Ok(())
/// }
/// ```
///
/// warp 0.3 example:
///
/// ```no_run
/// use warp::Filter;
/// use lambda_web::{is_running_on_lambda, run_hyper_on_lambda, LambdaError};
///
/// #[tokio::main]
/// async fn main() -> Result<(),LambdaError> {
///     // GET /hello/warp => 200 OK with body "Hello, warp!"
///     let hello = warp::path!("hello" / String)
///         .map(|name| format!("Hello, {}!", name));
///
///     if is_running_on_lambda() {
///         // Run app on AWS Lambda
///         run_hyper_on_lambda(warp::service(hello)).await?;
///     } else {
///         // Run app on local server
///         warp::serve(hello).run(([127, 0, 0, 1], 8080)).await;
///     }
///     Ok(())
/// }
/// ```
pub async fn run_hyper_on_lambda<S, B>(svc: S) -> Result<(), LambdaError>
where
    S: hyper::service::Service<HyperRequest, Response = HyperResponse<B>, Error = LambdaError>
        + 'static,
    B: hyper::body::HttpBody,
    <B as hyper::body::HttpBody>::Error: std::error::Error + Send + Sync + 'static,
{
    lambda_runtime::run(HyperHandler(svc)).await?;
    Ok(())
}

/// Lambda_runtime handler for hyper
struct HyperHandler<S, B>(S)
where
    S: hyper::service::Service<HyperRequest, Response = HyperResponse<B>> + 'static,
    B: hyper::body::HttpBody,
    <B as hyper::body::HttpBody>::Error: std::error::Error + Send + Sync + 'static;

impl<S, B> LambdaService<LambdaEvent<LambdaHttpEvent<'_>>> for HyperHandler<S, B>
where
    S: hyper::service::Service<HyperRequest, Response = HyperResponse<B>, Error = LambdaError>
        + 'static,
    B: hyper::body::HttpBody,
    <B as hyper::body::HttpBody>::Error: std::error::Error + Send + Sync + 'static,
{
    type Response = serde_json::Value;
    type Error = LambdaError;
    type Future = Pin<Box<dyn Future<Output = Result<serde_json::Value, Self::Error>>>>;

    /// Returns Poll::Ready when servie can process more requrests.
    fn poll_ready(
        &mut self,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    /// Lambda handler function
    /// Parse Lambda event as hyper request,
    /// serialize hyper response to Lambda JSON response
    fn call(&mut self, req: LambdaEvent<LambdaHttpEvent<'_>>) -> Self::Future {
        use serde_json::json;

        let event = req.payload;
        let _context = req.context;

        // check if web client supports content-encoding: br
        let client_br = event.client_supports_brotli();
        // multi-value-headers response format
        let multi_value = event.multi_value();

        // Parse request
        let hyper_request = HyperRequest::try_from(event);

        // Call hyper service when request parsing succeeded
        let svc_call = hyper_request.map(|req| self.0.call(req));

        let fut = async move {
            match svc_call {
                Ok(svc_fut) => {
                    // Request parsing succeeded
                    if let Ok(response) = svc_fut.await {
                        // Returns as API Gateway response
                        api_gateway_response_from_hyper(response, client_br, multi_value)
                            .await
                            .or_else(|_err| {
                                Ok(json!({
                                    "isBase64Encoded": false,
                                    "statusCode": 500u16,
                                    "headers": { "content-type": "text/plain"},
                                    "body": "Internal Server Error"
                                }))
                            })
                    } else {
                        // Some hyper error -> 500 Internal Server Error
                        Ok(json!({
                            "isBase64Encoded": false,
                            "statusCode": 500u16,
                            "headers": { "content-type": "text/plain"},
                            "body": "Internal Server Error"
                        }))
                    }
                }
                Err(_request_err) => {
                    // Request parsing error
                    Ok(json!({
                        "isBase64Encoded": false,
                        "statusCode": 400u16,
                        "headers": { "content-type": "text/plain"},
                        "body": "Bad Request"
                    }))
                }
            }
        };
        Box::pin(fut)
    }
}

impl TryFrom<LambdaHttpEvent<'_>> for HyperRequest {
    type Error = LambdaError;

    /// hyper Request from API Gateway event
    fn try_from(event: LambdaHttpEvent) -> Result<Self, Self::Error> {
        use hyper::header::{HeaderName, HeaderValue};
        use hyper::Method;
        use std::str::FromStr;

        // URI
        let uri = format!(
            "https://{}{}",
            event.hostname().unwrap_or("localhost"),
            event.path_query()
        );

        // Method
        let method = Method::try_from(event.method())?;

        // Construct hyper request
        let mut reqbuilder = hyper::Request::builder().method(method).uri(&uri);

        // headers
        if let Some(headers_mut) = reqbuilder.headers_mut() {
            for (k, v) in event.headers() {
                if let (Ok(k), Ok(v)) = (
                    HeaderName::from_str(k as &str),
                    HeaderValue::from_str(&v as &str),
                ) {
                    headers_mut.insert(k, v);
                }
            }
        }

        // Body
        let req = reqbuilder.body(hyper::Body::from(event.body()?))?;

        Ok(req)
    }
}

impl<B> crate::brotli::ResponseCompression for HyperResponse<B> {
    /// Content-Encoding header value
    fn content_encoding<'a>(&'a self) -> Option<&'a str> {
        self.headers()
            .get(hyper::header::CONTENT_ENCODING)
            .and_then(|val| val.to_str().ok())
    }

    /// Content-Type header value
    fn content_type<'a>(&'a self) -> Option<&'a str> {
        self.headers()
            .get(hyper::header::CONTENT_TYPE)
            .and_then(|val| val.to_str().ok())
    }
}

/// API Gateway response from hyper response
async fn api_gateway_response_from_hyper<B>(
    response: HyperResponse<B>,
    client_support_br: bool,
    multi_value: bool,
) -> Result<serde_json::Value, LambdaError>
where
    B: hyper::body::HttpBody,
    <B as hyper::body::HttpBody>::Error: std::error::Error + Send + Sync + 'static,
{
    use crate::brotli::ResponseCompression;
    use hyper::header::SET_COOKIE;
    use serde_json::json;

    // Check if response should be compressed
    let compress = client_support_br && response.can_brotli_compress();

    // Divide resonse into headers and body
    let (parts, res_body) = response.into_parts();

    // HTTP status
    let status_code = parts.status.as_u16();

    // Convert header to JSON map
    let mut cookies = Vec::<String>::new();
    let mut headers = serde_json::Map::new();
    for (k, v) in parts.headers.iter() {
        if let Ok(value_str) = v.to_str() {
            if multi_value {
                // REST API format, returns multiValueHeaders
                if let Some(values) = headers.get_mut(k.as_str()) {
                    if let Some(value_ary) = values.as_array_mut() {
                        value_ary.push(json!(value_str));
                    }
                } else {
                    headers.insert(k.as_str().to_string(), json!([value_str]));
                }
            } else {
                // HTTP API v2 format, returns headers
                if k == SET_COOKIE {
                    cookies.push(value_str.to_string());
                } else {
                    headers.insert(k.as_str().to_string(), json!(value_str));
                }
            }
        }
    }

    // Compress, base64 encode the response body
    let body_bytes = hyper::body::to_bytes(res_body).await?;
    let body_base64 = if compress {
        if multi_value {
            headers.insert("content-encoding".to_string(), json!(["br"]));
        } else {
            headers.insert("content-encoding".to_string(), json!("br"));
        }
        crate::brotli::compress_response_body(&body_bytes)
    } else {
        base64::encode(body_bytes)
    };

    if multi_value {
        Ok(json!({
            "isBase64Encoded": true,
            "statusCode": status_code,
            "multiValueHeaders": headers,
            "body": body_base64
        }))
    } else {
        Ok(json!({
            "isBase64Encoded": true,
            "statusCode": status_code,
            "cookies": cookies,
            "headers": headers,
            "body": body_base64
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{request::LambdaHttpEvent, test_consts::*};

    // Request JSON string to http::Request
    fn prepare_request(event_str: &str) -> HyperRequest {
        let reqjson: LambdaHttpEvent = serde_json::from_str(event_str).unwrap();
        let req = HyperRequest::try_from(reqjson).unwrap();
        req
    }

    #[test]
    fn test_path_decode() {
        let req = prepare_request(API_GATEWAY_V2_GET_ROOT_NOQUERY);
        assert_eq!(req.uri().path(), "/");
        let req = prepare_request(API_GATEWAY_REST_GET_ROOT_NOQUERY);
        assert_eq!(req.uri().path(), "/stage/");

        let req = prepare_request(API_GATEWAY_V2_GET_SOMEWHERE_NOQUERY);
        assert_eq!(req.uri().path(), "/somewhere");
        let req = prepare_request(API_GATEWAY_REST_GET_SOMEWHERE_NOQUERY);
        assert_eq!(req.uri().path(), "/stage/somewhere");

        let req = prepare_request(API_GATEWAY_V2_GET_SPACEPATH_NOQUERY);
        assert_eq!(req.uri().path(), "/path%20with/space");
        let req = prepare_request(API_GATEWAY_REST_GET_SPACEPATH_NOQUERY);
        assert_eq!(req.uri().path(), "/stage/path%20with/space");

        let req = prepare_request(API_GATEWAY_V2_GET_PERCENTPATH_NOQUERY);
        assert_eq!(req.uri().path(), "/path%25with/percent");
        let req = prepare_request(API_GATEWAY_REST_GET_PERCENTPATH_NOQUERY);
        assert_eq!(req.uri().path(), "/stage/path%25with/percent");

        let req = prepare_request(API_GATEWAY_V2_GET_UTF8PATH_NOQUERY);
        assert_eq!(
            req.uri().path(),
            "/%E6%97%A5%E6%9C%AC%E8%AA%9E/%E3%83%95%E3%82%A1%E3%82%A4%E3%83%AB%E5%90%8D"
        );
        let req = prepare_request(API_GATEWAY_REST_GET_UTF8PATH_NOQUERY);
        assert_eq!(
            req.uri().path(),
            "/stage/%E6%97%A5%E6%9C%AC%E8%AA%9E/%E3%83%95%E3%82%A1%E3%82%A4%E3%83%AB%E5%90%8D"
        );
    }

    #[test]
    fn test_query_decode() {
        let req = prepare_request(API_GATEWAY_V2_GET_ROOT_ONEQUERY);
        assert_eq!(req.uri().query(), Some("key=value"));
        let req = prepare_request(API_GATEWAY_REST_GET_ROOT_ONEQUERY);
        assert_eq!(req.uri().query(), Some("key=value"));

        let req = prepare_request(API_GATEWAY_V2_GET_SOMEWHERE_ONEQUERY);
        assert_eq!(req.uri().query(), Some("key=value"));
        let req = prepare_request(API_GATEWAY_REST_GET_SOMEWHERE_ONEQUERY);
        assert_eq!(req.uri().query(), Some("key=value"));

        let req = prepare_request(API_GATEWAY_V2_GET_SOMEWHERE_TWOQUERY);
        assert_eq!(req.uri().query(), Some("key1=value1&key2=value2"));
        let req = prepare_request(API_GATEWAY_REST_GET_SOMEWHERE_TWOQUERY);
        assert!(
            req.uri().query() == Some("key1=value1&key2=value2")
                || req.uri().query() == Some("key2=value2&key1=value1")
        );

        let req = prepare_request(API_GATEWAY_V2_GET_SOMEWHERE_SPACEQUERY);
        assert_eq!(req.uri().query(), Some("key=value1+value2"));
        let req = prepare_request(API_GATEWAY_REST_GET_SOMEWHERE_SPACEQUERY);
        assert_eq!(req.uri().query(), Some("key=value1%20value2"));

        let req = prepare_request(API_GATEWAY_V2_GET_SOMEWHERE_UTF8QUERY);
        assert_eq!(req.uri().query(), Some("key=%E6%97%A5%E6%9C%AC%E8%AA%9E"));
        let req = prepare_request(API_GATEWAY_REST_GET_SOMEWHERE_UTF8QUERY);
        assert_eq!(req.uri().query(), Some("key=%E6%97%A5%E6%9C%AC%E8%AA%9E"));
    }

    #[tokio::test]
    async fn test_form_post() {
        use hyper::body::to_bytes;
        use hyper::Method;

        let req = prepare_request(API_GATEWAY_V2_POST_FORM_URLENCODED);
        assert_eq!(req.method(), Method::POST);
        assert_eq!(
            to_bytes(req.into_body()).await.unwrap().as_ref(),
            b"key1=value1&key2=value2&Ok=Ok"
        );
        let req = prepare_request(API_GATEWAY_REST_POST_FORM_URLENCODED);
        assert_eq!(req.method(), Method::POST);
        assert_eq!(
            to_bytes(req.into_body()).await.unwrap().as_ref(),
            b"key1=value1&key2=value2&Ok=Ok"
        );

        // Base64 encoded
        let req = prepare_request(API_GATEWAY_V2_POST_FORM_URLENCODED_B64);
        assert_eq!(req.method(), Method::POST);
        assert_eq!(
            to_bytes(req.into_body()).await.unwrap().as_ref(),
            b"key1=value1&key2=value2&Ok=Ok"
        );
        let req = prepare_request(API_GATEWAY_REST_POST_FORM_URLENCODED_B64);
        assert_eq!(req.method(), Method::POST);
        assert_eq!(
            to_bytes(req.into_body()).await.unwrap().as_ref(),
            b"key1=value1&key2=value2&Ok=Ok"
        );
    }

    #[test]
    fn test_parse_header() {
        let req = prepare_request(API_GATEWAY_V2_GET_ROOT_NOQUERY);
        assert_eq!(req.headers().get("x-forwarded-port").unwrap(), &"443");
        assert_eq!(req.headers().get("x-forwarded-proto").unwrap(), &"https");
        let req = prepare_request(API_GATEWAY_REST_GET_ROOT_NOQUERY);
        assert_eq!(req.headers().get("x-forwarded-port").unwrap(), &"443");
        assert_eq!(req.headers().get("x-forwarded-proto").unwrap(), &"https");
    }

    #[test]
    fn test_parse_cookies() {
        let req = prepare_request(API_GATEWAY_V2_GET_ROOT_NOQUERY);
        assert_eq!(req.headers().get("cookie"), None);
        let req = prepare_request(API_GATEWAY_REST_GET_ROOT_NOQUERY);
        assert_eq!(req.headers().get("cookie"), None);

        let req = prepare_request(API_GATEWAY_V2_GET_ONE_COOKIE);
        assert_eq!(req.headers().get("cookie").unwrap(), &"cookie1=value1");
        let req = prepare_request(API_GATEWAY_REST_GET_ONE_COOKIE);
        assert_eq!(req.headers().get("cookie").unwrap(), &"cookie1=value1");

        let req = prepare_request(API_GATEWAY_V2_GET_TWO_COOKIES);
        assert_eq!(
            req.headers().get("cookie").unwrap(),
            &"cookie1=value1; cookie2=value2"
        );
        let req = prepare_request(API_GATEWAY_REST_GET_TWO_COOKIES);
        assert_eq!(
            req.headers().get("cookie").unwrap(),
            &"cookie1=value1; cookie2=value2"
        );
    }
}
