// SPDX-License-Identifier: MIT
//!
//! Run Rocket on AWS Lambda
//!
//!
use crate::request::LambdaHttpEvent;
use core::convert::TryFrom;
use core::future::Future;
use lambda_runtime::{
    run as lambda_runtime_run, Context as LambdaContext, Error as LambdaError,
    Handler as LambdaHandler,
};
use std::pin::Pin;
use std::sync::Arc;

/// Launch Rocket application on AWS Lambda
///
/// ```no_run
/// use rocket::{self, get, routes};
/// use lambda_web::{is_running_on_lambda, launch_rocket_on_lambda, LambdaError};
///
/// #[get("/hello/<name>/<age>")]
/// fn hello(name: &str, age: u8) -> String {
///     format!("Hello, {} year old named {}!", age, name)
/// }
///
/// #[rocket::main]
/// async fn main() -> Result<(), LambdaError> {
///     let rocket = rocket::build().mount("/", routes![hello]);
///     if is_running_on_lambda() {
///         // Launch on AWS Lambda
///         launch_rocket_on_lambda(rocket).await?;
///     } else {
///         // Launch local server
///         rocket.launch().await?;
///     }
///     Ok(())
/// }
/// ```
///
pub async fn launch_rocket_on_lambda<P: rocket::Phase>(
    r: rocket::Rocket<P>,
) -> Result<(), LambdaError> {
    lambda_runtime_run(RocketHandler(Arc::new(
        rocket::local::asynchronous::Client::untracked(r).await?,
    )))
    .await?;

    Ok(())
}

/// Lambda_runtime handler for Rocket
struct RocketHandler(Arc<rocket::local::asynchronous::Client>);

impl LambdaHandler<LambdaHttpEvent<'_>, serde_json::Value> for RocketHandler {
    type Error = rocket::Error;
    type Fut = Pin<Box<dyn Future<Output = Result<serde_json::Value, Self::Error>> + Send>>;

    /// Lambda handler function
    /// Parse Lambda event as Rocket LocalRequest,
    /// serialize Rocket LocalResponse to Lambda JSON response
    fn call(&self, event: LambdaHttpEvent, _context: LambdaContext) -> Self::Fut {
        use serde_json::json;

        // check if web client supports content-encoding: br
        let client_br = event.client_supports_brotli();

        // Parse request
        let decode_result = RequestDecode::try_from(event);
        let client = self.0.clone();
        let fut = async move {
            match decode_result {
                Ok(req_decode) => {
                    // Request parsing succeeded, make Rocket LocalRequest
                    let local_request = req_decode.make_request(&client);

                    // Dispatch request and get response
                    let response = local_request.dispatch().await;

                    // Return response as API Gateway JSON
                    api_gateway_response_from_rocket(response, client_br).await
                }
                Err(_request_err) => {
                    // Request parsing error
                    Ok(json!({
                        "isBase64Encoded": false,
                        "statusCode": 400u16,
                        "headers": { "content-type": "text/plain"},
                        "body": "Bad Request" // No details for security
                    }))
                }
            }
        };
        Box::pin(fut)
    }
}

// Request decoded from API gateway JSON.
// To move async boundary in call() function,
// all elements must be owned
struct RequestDecode {
    path_and_query: String,
    method: rocket::http::Method,
    source_ip: std::net::IpAddr,
    cookies: Vec<String>,
    headers: Vec<rocket::http::Header<'static>>,
    body: Vec<u8>,
}

impl TryFrom<LambdaHttpEvent<'_>> for RequestDecode {
    type Error = LambdaError;

    /// Request from API Gateway event
    fn try_from(event: LambdaHttpEvent) -> Result<Self, Self::Error> {
        use rocket::http::{Header, Method};
        use std::net::IpAddr;
        use std::str::FromStr;

        // path ? query_string
        let path_and_query = event.path_query();

        // Method, Source IP
        let method = Method::from_str(&event.method()).map_err(|_| "InvalidMethod")?;
        let source_ip = event
            .source_ip()
            .unwrap_or(IpAddr::from([0u8, 0u8, 0u8, 0u8]));

        // Parse cookies
        let cookies = event.cookies().iter().map(|c| c.to_string()).collect();

        // Headers
        let headers = event
            .headers()
            .iter()
            .map(|(k, v)| Header::new(k.to_string(), v.to_string()))
            .collect::<Vec<Header>>();

        // Body
        let body = event.body()?;

        Ok(Self {
            path_and_query,
            method,
            source_ip,
            cookies,
            headers,
            body,
        })
    }
}

impl RequestDecode {
    /// Make Rocket LocalRequest
    fn make_request<'c, 's: 'c>(
        &'s self,
        client: &'c rocket::local::asynchronous::Client,
    ) -> rocket::local::asynchronous::LocalRequest<'c> {
        use rocket::http::Cookie;

        // path, method, remote address, body
        let req = client
            .req(self.method, &self.path_and_query)
            .remote(std::net::SocketAddr::from((self.source_ip, 0u16)))
            .body(&self.body);

        // Copy cookies
        let req = self.cookies.iter().fold(req, |req, cookie_name_val| {
            if let Ok(cookie) = Cookie::parse_encoded(cookie_name_val) {
                req.cookie(cookie)
            } else {
                req
            }
        });

        // Copy headers
        let req = self
            .headers
            .iter()
            .fold(req, |req, header| req.header(header.clone()));

        req
    }
}

impl crate::brotli::ResponseCompression for rocket::local::asynchronous::LocalResponse<'_> {
    /// Content-Encoding header value
    fn content_encoding<'a>(&'a self) -> Option<&'a str> {
        self.headers().get_one("content-encoding")
    }

    /// Content-Type header value
    fn content_type<'a>(&'a self) -> Option<&'a str> {
        self.headers().get_one("content-type")
    }
}

/// API Gateway response from Rocket response
async fn api_gateway_response_from_rocket(
    response: rocket::local::asynchronous::LocalResponse<'_>,
    client_support_br: bool,
) -> Result<serde_json::Value, rocket::Error> {
    use crate::brotli::ResponseCompression;
    use serde_json::json;

    // HTTP status
    let status_code = response.status().code;

    // Convert header to JSON map
    let mut headers = serde_json::Map::new();
    for header in response.headers().iter() {
        headers.insert(header.name.into_string(), json!(header.value));
    }

    // check if response should be compressed
    let compress = client_support_br && response.can_brotli_compress();
    let body_bytes = response.into_bytes().await.unwrap_or_default();
    let body_base64 = if compress {
        headers.insert("content-encoding".to_string(), json!("br"));
        crate::brotli::compress_response_body(&body_bytes)
    } else {
        base64::encode(body_bytes)
    };

    Ok(json!({
        "isBase64Encoded": true,
        "statusCode": status_code,
        "headers": headers,
        "body": body_base64
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{request::LambdaHttpEvent, test_consts::*};
    use rocket::{async_test, local::asynchronous::Client};
    use std::path::PathBuf;

    // Request JSON to actix_http::Request
    fn prepare_request(event_str: &str) -> RequestDecode {
        let reqjson: LambdaHttpEvent = serde_json::from_str(event_str).unwrap();
        let decode = RequestDecode::try_from(reqjson).unwrap();
        decode
    }

    #[async_test]
    async fn test_path_decode() {
        let rocket = rocket::build();
        let client = Client::untracked(rocket).await.unwrap();

        let decode = prepare_request(API_GATEWAY_V2_GET_ROOT_NOQUERY);
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/");
        assert_eq!(req.inner().segments(0..), Ok(PathBuf::new()));

        let decode = prepare_request(API_GATEWAY_V2_GET_SOMEWHERE_NOQUERY);
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/somewhere");
        assert_eq!(req.inner().segments(0..), Ok(PathBuf::from("somewhere")));

        let decode = prepare_request(API_GATEWAY_V2_GET_SPACEPATH_NOQUERY);
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/path%20with/space");
        assert_eq!(
            req.inner().segments(0..),
            Ok(PathBuf::from("path with/space"))
        );

        let decode = prepare_request(API_GATEWAY_V2_GET_PERCENTPATH_NOQUERY);
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/path%25with/percent");
        assert_eq!(
            req.inner().segments(0..),
            Ok(PathBuf::from("path%with/percent"))
        );

        let decode = prepare_request(API_GATEWAY_V2_GET_UTF8PATH_NOQUERY);
        let req = decode.make_request(&client);
        assert_eq!(
            &decode.path_and_query,
            "/%E6%97%A5%E6%9C%AC%E8%AA%9E/%E3%83%95%E3%82%A1%E3%82%A4%E3%83%AB%E5%90%8D"
        );
        assert_eq!(
            req.inner().segments(0..),
            Ok(PathBuf::from("日本語/ファイル名"))
        );
    }

    #[async_test]
    async fn test_query_decode() {
        let rocket = rocket::build();
        let client = Client::untracked(rocket).await.unwrap();

        let decode = prepare_request(API_GATEWAY_V2_GET_ROOT_ONEQUERY);
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/?key=value");
        assert_eq!(req.inner().segments(0..), Ok(PathBuf::new()));
        assert_eq!(req.inner().query_value::<&str>("key").unwrap(), Ok("value"));

        let decode = prepare_request(API_GATEWAY_V2_GET_SOMEWHERE_ONEQUERY);
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/somewhere?key=value");
        assert_eq!(req.inner().segments(0..), Ok(PathBuf::from("somewhere")));
        assert_eq!(req.inner().query_value::<&str>("key").unwrap(), Ok("value"));

        let decode = prepare_request(API_GATEWAY_V2_GET_SOMEWHERE_TWOQUERY);
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/somewhere?key1=value1&key2=value2");
        assert_eq!(
            req.inner().query_value::<&str>("key1").unwrap(),
            Ok("value1")
        );
        assert_eq!(
            req.inner().query_value::<&str>("key2").unwrap(),
            Ok("value2")
        );

        let decode = prepare_request(API_GATEWAY_V2_GET_SOMEWHERE_SPACEQUERY);
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/somewhere?key=value1+value2");
        assert_eq!(
            req.inner().query_value::<&str>("key").unwrap(),
            Ok("value1 value2")
        );

        let decode = prepare_request(API_GATEWAY_V2_GET_SOMEWHERE_UTF8QUERY);
        let req = decode.make_request(&client);
        assert_eq!(
            &decode.path_and_query,
            "/somewhere?key=%E6%97%A5%E6%9C%AC%E8%AA%9E"
        );
        assert_eq!(
            req.inner().query_value::<&str>("key").unwrap(),
            Ok("日本語")
        );
    }

    #[async_test]
    async fn test_remote_ip_decode() {
        use std::net::IpAddr;
        use std::str::FromStr;

        let rocket = rocket::build();
        let client = Client::untracked(rocket).await.unwrap();

        let decode = prepare_request(API_GATEWAY_V2_GET_ROOT_ONEQUERY);
        let req = decode.make_request(&client);
        assert_eq!(decode.source_ip, IpAddr::from_str("1.2.3.4").unwrap());
        assert_eq!(
            req.inner().client_ip(),
            Some(IpAddr::from_str("1.2.3.4").unwrap())
        );

        let decode = prepare_request(API_GATEWAY_V2_GET_REMOTE_IPV6);
        let req = decode.make_request(&client);
        assert_eq!(
            decode.source_ip,
            IpAddr::from_str("2404:6800:400a:80c::2004").unwrap()
        );
        assert_eq!(
            req.inner().client_ip(),
            Some(IpAddr::from_str("2404:6800:400a:80c::2004").unwrap())
        );
    }

    #[async_test]
    async fn test_form_post() {
        use rocket::http::ContentType;
        use rocket::http::Method;
        let rocket = rocket::build();
        let client = Client::untracked(rocket).await.unwrap();

        let decode = prepare_request(API_GATEWAY_V2_POST_FORM_URLENCODED);
        let req = decode.make_request(&client);
        assert_eq!(&decode.body, b"key1=value1&key2=value2&Ok=Ok");
        assert_eq!(req.inner().method(), Method::Post);
        assert_eq!(req.inner().content_type(), Some(&ContentType::Form));

        // Base64 encoded
        let decode = prepare_request(API_GATEWAY_V2_POST_FORM_URLENCODED_B64);
        let req = decode.make_request(&client);
        assert_eq!(&decode.body, b"key1=value1&key2=value2&Ok=Ok");
        assert_eq!(req.inner().method(), Method::Post);
        assert_eq!(req.inner().content_type(), Some(&ContentType::Form));
    }

    #[async_test]
    async fn test_parse_header() {
        let rocket = rocket::build();
        let client = Client::untracked(rocket).await.unwrap();

        let decode = prepare_request(API_GATEWAY_V2_GET_ROOT_NOQUERY);
        let req = decode.make_request(&client);
        assert_eq!(
            req.inner().headers().get_one("x-forwarded-port"),
            Some("443")
        );
        assert_eq!(
            req.inner().headers().get_one("x-forwarded-proto"),
            Some("https")
        );
    }

    #[async_test]
    async fn test_parse_cookies() {
        let rocket = rocket::build();
        let client = Client::untracked(rocket).await.unwrap();

        let decode = prepare_request(API_GATEWAY_V2_GET_ROOT_NOQUERY);
        let req = decode.make_request(&client);
        assert_eq!(req.inner().cookies().iter().count(), 0);

        let decode = prepare_request(API_GATEWAY_V2_GET_ONE_COOKIE);
        let req = decode.make_request(&client);
        assert_eq!(
            req.inner().cookies().get("cookie1").unwrap().value(),
            "value1"
        );

        let decode = prepare_request(API_GATEWAY_V2_GET_TWO_COOKIES);
        let req = decode.make_request(&client);
        assert_eq!(
            req.inner().cookies().get("cookie1").unwrap().value(),
            "value1"
        );
        assert_eq!(
            req.inner().cookies().get("cookie2").unwrap().value(),
            "value2"
        );
    }
}
