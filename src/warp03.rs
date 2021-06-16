// SPDX-License-Identifier: MIT
//!
//! Run Warp 0.3.x on AWS Lambda
//!
use crate::request::ApiGatewayV2;
use core::convert::TryFrom;
use core::future::Future;
use std::convert::Infallible;
use std::pin::Pin;
/*
use lambda_runtime::{
    run as lambda_runtime_run, Context as LambdaContext, Error as LambdaError,
    Handler as LambdaHandler,
};
*/
use lamedh_runtime::{
    run as lambda_runtime_run, Context as LambdaContext, Error as LambdaError,
    Handler as LambdaHandler,
};

type WarpRequest = warp::http::Request<warp::hyper::Body>;
type WarpResponse = warp::http::Response<warp::hyper::Body>;

/// Run Warp application on AWS Lambda
///
/// ```
/// use lambda_web::warp::{self, Filter};
/// use lambda_web::{is_running_on_lambda, run_warp_on_lambda, LambdaError};
///
/// #[tokio::main]
/// async fn main() -> Result<(),LambdaError> {
///     // GET /hello/warp => 200 OK with body "Hello, warp!"
///     let hello = warp::path!("hello" / String)
///         .map(|name| format!("Hello, {}!", name));
///
///     if is_running_on_lambda() {
///         // Run on AWS Lambda
///         run_warp_on_lambda(warp::service(hello)).await?;
///     } else {
///         // Run local server
///         warp::serve(hello)
///             .run(([127, 0, 0, 1], 8080))
///             .await;
///     }
///     Ok(())
/// }
/// ```
///
pub async fn run_warp_on_lambda<S>(svc: S) -> Result<(), LambdaError>
where
    S: warp::hyper::service::Service<WarpRequest, Response = WarpResponse, Error = Infallible>
        + Clone
        + Send
        + 'static,
    S::Future: Send,
{
    lambda_runtime_run(WarpHandler(svc)).await?;

    Ok(())
}

/// Lambda_runtime handler for Warp
struct WarpHandler<S>(S)
where
    S: warp::hyper::service::Service<WarpRequest, Response = WarpResponse, Error = Infallible>
        + 'static;

impl<S> LambdaHandler<ApiGatewayV2<'_>, serde_json::Value> for WarpHandler<S>
where
    S: warp::hyper::service::Service<WarpRequest, Response = WarpResponse, Error = Infallible>
        + 'static,
{
    type Error = LambdaError;
    type Fut = Pin<Box<dyn Future<Output = Result<serde_json::Value, Self::Error>> + 'static>>;

    /// Lambda handler function
    /// Parse Lambda event as Warp request,
    /// serialize Warp response to Lambda JSON response
    fn call(&mut self, event: ApiGatewayV2, _context: LambdaContext) -> Self::Fut {
        use serde_json::json;

        // Parse request
        let warp_request = WarpRequest::try_from(event);

        // Call Warp service when request parsing succeeded
        let svc_call = warp_request.map(|req| self.0.call(req));

        let fut = async move {
            match svc_call {
                Ok(svc_fut) => {
                    // Request parsing succeeded
                    if let Ok(response) = svc_fut.await {
                        // Returns as API Gateway response
                        api_gateway_response_from_warp(response).await
                    } else {
                        // Some Warp error -> 500 Internal Server Error
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

impl TryFrom<ApiGatewayV2<'_>> for WarpRequest {
    type Error = LambdaError;

    /// Warp Request from API Gateway event
    fn try_from(event: ApiGatewayV2) -> Result<Self, Self::Error> {
        use std::str::FromStr;
        use warp::http::header::{HeaderName, HeaderValue, COOKIE};
        use warp::http::Method;

        // URI
        let uri = if event.raw_query_string.is_empty() {
            format!(
                "https://{}{}",
                event.request_context.domain_name,
                event.encoded_path()
            )
        } else {
            format!(
                "https://{}{}?{}",
                event.request_context.domain_name,
                event.encoded_path(),
                event.raw_query_string
            )
        };

        // Method
        let method = Method::try_from(&event.request_context.http.method as &str)?;

        // Construct warp request
        let mut reqbuilder = warp::http::Request::builder().method(method).uri(&uri);

        // headers
        if let Some(headers_mut) = reqbuilder.headers_mut() {
            for (k, v) in &event.headers {
                if let (Ok(k), Ok(v)) = (
                    HeaderName::from_str(k as &str),
                    HeaderValue::from_str(v as &str),
                ) {
                    headers_mut.insert(k, v);
                }
            }
            // Cookies
            if let Some(cookies) = event.cookies {
                if let Ok(cookie_value) = HeaderValue::from_str(&cookies.join(";")) {
                    headers_mut.insert(COOKIE, cookie_value);
                }
            }
        }

        // Body
        let req = if let Some(eventbody) = event.body {
            if event.is_base64_encoded {
                // base64 decode
                let binarybody = base64::decode(&eventbody as &str)?;
                reqbuilder.body(warp::hyper::Body::from(binarybody))?
            } else {
                reqbuilder.body(warp::hyper::Body::from(eventbody.into_owned()))?
            }
        } else {
            reqbuilder.body(warp::hyper::Body::empty())?
        };

        Ok(req)
    }
}

/// API Gateway response from Warp response
async fn api_gateway_response_from_warp(
    response: WarpResponse,
) -> Result<serde_json::Value, LambdaError> {
    use serde_json::json;

    let (parts, res_body) = response.into_parts();

    // HTTP status
    let status_code = parts.status.as_u16();

    // Convert header to JSON map
    let mut headers = serde_json::Map::new();
    for (k, v) in parts.headers.iter() {
        if let Ok(value_str) = v.to_str() {
            headers.insert(k.as_str().to_string(), json!(value_str));
        }
    }

    // Body
    let body_bytes = warp::hyper::body::to_bytes(res_body).await?;

    Ok(json!({
        "isBase64Encoded": true,
        "statusCode": status_code,
        "headers": headers,
        "body": base64::encode(body_bytes.to_vec())
    }))
}
