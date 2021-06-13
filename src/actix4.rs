//!
//! Run Actix Web on AWS Lambda
//!
//!
use crate::request::ApiGatewayV2;
use core::convert::TryFrom;
use core::future::Future;
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

/// Run Actix-web application on AWS Lambda
///
/// ```
/// #[get("/{id}/{name}/index.html")]
/// async fn index(path: web::Path<(u32, String)>) -> impl Responder {
///     let (id, name) = path.into_inner();
///     format!("Hello {}! id:{}", name, id)
/// }
///
/// #[actix_web::main]
/// async fn main() -> Result<(),LambdaError> {
///     let factory = move || {
///         App::new().service(index)
///     };
///     if is_running_on_lambda() {
///         run_actix_on_lambda(factory).await?
///     } else {
///         // Run local server
///         HttpServer::new(factory).bind("127.0.0.1:8080")?.run().await?;
///     }
/// }
/// ```
///
pub async fn run_actix_on_lambda<F, I, S, B>(factory: F) -> Result<(), LambdaError>
where
    F: Fn() -> I + Send + Clone + 'static,
    I: actix_service::IntoServiceFactory<S, actix_http::Request>,
    S: actix_service::ServiceFactory<
            actix_http::Request,
            Config = actix_web::dev::AppConfig,
            Response = actix_web::dev::ServiceResponse<B>,
            Error = actix_web::Error,
        > + 'static,
    S::InitError: std::fmt::Debug,
    B: actix_web::body::MessageBody,
{
    // Prepare actix_service::Service
    let srv = factory().into_factory();
    let new_svc = srv
        .new_service(actix_web::dev::AppConfig::default())
        .await
        .unwrap();

    lambda_runtime_run(ActixHandler(new_svc)).await?;

    Ok(())
}

/// Lambda_runtime handler for Actix Web
struct ActixHandler<S, B>(S)
where
    S: actix_service::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse<B>,
            Error = actix_web::Error,
        > + 'static,
    B: actix_web::body::MessageBody;

impl<S, B> LambdaHandler<ApiGatewayV2<'_>, serde_json::Value> for ActixHandler<S, B>
where
    S: actix_service::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse<B>,
            Error = actix_web::Error,
        > + 'static,
    B: actix_web::body::MessageBody,
{
    type Error = actix_web::Error;
    type Fut = Pin<Box<dyn Future<Output = Result<serde_json::Value, Self::Error>> + 'static>>;

    /// Lambda handler function
    /// Parse Lambda event as Actix-web request,
    /// serialize Actix-web response to Lambda JSON response
    fn call(&mut self, event: ApiGatewayV2, _context: LambdaContext) -> Self::Fut {
        use serde_json::json;

        // Parse request
        let actix_request = actix_http::Request::try_from(event);

        // Call Actix service when request parsing succeeded
        let svc_call = actix_request.map(|req| self.0.call(req));

        let fut = async move {
            match svc_call {
                Ok(svc_fut) => {
                    // Request parsing succeeded
                    if let Ok(response) = svc_fut.await {
                        // Returns as API Gateway response
                        api_gateway_response_from_actix_web(response).await
                    } else {
                        // Some Actix web error -> 500 Internal Server Error
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

impl TryFrom<ApiGatewayV2<'_>> for actix_http::Request {
    type Error = LambdaError;

    /// Actix-web Request from API Gateway event
    fn try_from(event: ApiGatewayV2) -> Result<Self, Self::Error> {
        use actix_web::cookie::Cookie;
        use actix_web::http::Method;
        use std::borrow::Cow;
        use std::net::IpAddr;
        use std::str::FromStr;

        // path ? query_string
        let path_and_query: Cow<str> = if event.raw_query_string.is_empty() {
            event.raw_path
        } else {
            format!("{}?{}", event.raw_path, event.raw_query_string).into()
        };

        // Method, Source IP
        let method = Method::try_from(&event.request_context.http.method as &str)?;
        let source_ip = IpAddr::from_str(&event.request_context.http.source_ip as &str)?;

        // Construct actix_web request
        let req = actix_web::test::TestRequest::with_uri(&path_and_query)
            .method(method)
            .peer_addr(std::net::SocketAddr::from((source_ip, 0u16)));

        // Cookies
        let req = if let Some(cookies) = event.cookies {
            cookies.iter().fold(req, |req, cookie| {
                if let Ok(cookie_decoded) = Cookie::parse_encoded(cookie as &str) {
                    req.cookie(cookie_decoded)
                } else {
                    req
                }
            })
        } else {
            req
        };

        // Headers
        let req = event
            .headers
            .iter()
            .fold(req, |req, (k, v)| req.insert_header((k as &str, v as &str)));

        // Body
        let req = if let Some(eventbody) = event.body {
            if event.is_base64_encoded {
                // base64 decode
                let binarybody = base64::decode(&eventbody as &str)?;
                req.set_payload(binarybody)
            } else {
                req.set_payload((&eventbody as &str).to_string())
            }
        } else {
            req
        };

        Ok(req.to_request())
    }
}

/// API Gateway response from Actix-web response
async fn api_gateway_response_from_actix_web<B: actix_web::body::MessageBody>(
    mut response: actix_web::dev::ServiceResponse<B>,
) -> Result<serde_json::Value, actix_web::Error> {
    use serde_json::json;

    // HTTP status
    let status_code = response.status().as_u16();

    // Convert header to JSON map
    let mut headers = serde_json::Map::new();
    for (k, v) in response.headers() {
        if let Ok(value_str) = v.to_str() {
            headers.insert(k.as_str().to_string(), json!(value_str));
        }
    }

    // Body
    let body_bytes = actix_web::body::to_bytes(response.take_body()).await?;

    Ok(json!({
        "isBase64Encoded": true,
        "statusCode": status_code,
        "headers": headers,
        "body": base64::encode(body_bytes.to_vec())
    }))
}
