// SPDX-License-Identifier: MIT
//!
//! Run Rocket on AWS Lambda
//!
//!
use crate::request::LambdaHttpEvent;
use core::convert::TryFrom;
use core::future::Future;
use lambda_runtime::{Error as LambdaError, LambdaEvent, Service as LambdaService};
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
    // println!("launched rocket");

    // let thing = r#"{"body":null,"cookies":null,"headers":{"accept":"text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7","accept-encoding":"gzip, deflate, br","accept-language":"en-US,en;q=0.9","cache-control":"max-age=0","connection":"keep-alive","host":"localhost:9000","lambda-runtime-aws-request-id":"8c639787-10ef-4e7f-96e9-148ce877b284","sec-ch-ua":"\"Chromium\";v=\"110\", \"Not A(Brand\";v=\"24\", \"Google Chrome\";v=\"110\"","sec-ch-ua-mobile":"?0","sec-ch-ua-platform":"\"macOS\"","sec-fetch-dest":"document","sec-fetch-mode":"navigate","sec-fetch-site":"none","sec-fetch-user":"?1","upgrade-insecure-requests":"1","user-agent":"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36"},"isBase64Encoded":false,"pathParameters":{},"queryStringParameters":{},"rawPath":"/","rawQueryString":null,"requestContext":{"accountId":null,"apiId":null,"authentication":null,"authorizer":null,"domainName":"localhost","domainPrefix":"_","http":{"method":"GET","path":"/","protocol":"http","sourceIp":"127.0.0.1","userAgent":"cargo-lambda"},"requestId":"8c639787-10ef-4e7f-96e9-148ce877b284","routeKey":"$default","stage":"$default","time":"19/May/2023:03:14:42 +0000","timeEpoch":1684466082},"routeKey":"$default","stageVariables":{},"version":"2.0"}"#;
    // let parsed_thing: Result<ApiGatewayHttpV2Event, _> = serde_json::from_str(thing);
    // println!("parsedThing: {:?}", parsed_thing);


    let handler: RocketHandler = RocketHandler(Arc::new(
        rocket::local::asynchronous::Client::untracked(r).await?,
    ));
    lambda_runtime::run(handler)
    .await?;

    Ok(())
}

/// Lambda_runtime handler for Rocket
struct RocketHandler(Arc<rocket::local::asynchronous::Client>);

impl LambdaService<LambdaEvent<LambdaHttpEvent<'_>>> for RocketHandler {
    type Response = serde_json::Value;
    type Error = rocket::Error;
    type Future = Pin<Box<dyn Future<Output = Result<serde_json::Value, Self::Error>> + Send>>;

    /// Always ready in case of Rocket local client
    fn poll_ready(
        &mut self,
        _cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<(), Self::Error>> {
        core::task::Poll::Ready(Ok(()))
    }

    /// Lambda handler function
    /// Parse Lambda event as Rocket LocalRequest,
    /// serialize Rocket LocalResponse to Lambda JSON response
    fn call(&mut self, req: LambdaEvent<LambdaHttpEvent<'_>>) -> Self::Future {
        println!("called lambda");
        use serde_json::json;

        let event = req.payload;
        let _context = req.context;

        // check if web client supports content-encoding: br
        let client_br = event.client_supports_brotli();
        // multi-value-headers response format
        let multi_value = event.multi_value();

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
                    api_gateway_response_from_rocket(response, client_br, multi_value).await
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
    multi_value: bool,
) -> Result<serde_json::Value, rocket::Error> {
    use crate::brotli::ResponseCompression;
    use serde_json::json;

    // HTTP status
    let status_code = response.status().code;

    // Convert header to JSON map
    let mut cookies = Vec::<String>::new();
    let mut headers = serde_json::Map::new();
    for header in response.headers().iter() {
        let header_name = header.name.into_string();
        let header_value = header.value.into_owned();
        if multi_value {
            // REST API format, returns multiValueHeaders
            if let Some(values) = headers.get_mut(&header_name) {
                if let Some(value_ary) = values.as_array_mut() {
                    value_ary.push(json!(header_value));
                }
            } else {
                headers.insert(header_name, json!([header_value]));
            }
        } else {
            // HTTP API v2 format, returns headers
            if &header_name == "set-cookie" {
                cookies.push(header_value);
            } else {
                headers.insert(header_name, json!(header_value));
            }
        }
    }

    // check if response should be compressed
    let compress = client_support_br && response.can_brotli_compress();
    let body_bytes = response.into_bytes().await.unwrap_or_default();
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
        let decode = prepare_request(API_GATEWAY_REST_GET_ROOT_NOQUERY);
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/stage/");
        assert_eq!(req.inner().segments(0..), Ok(PathBuf::from("stage")));

        let decode = prepare_request(API_GATEWAY_V2_GET_SOMEWHERE_NOQUERY);
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/somewhere");
        assert_eq!(req.inner().segments(0..), Ok(PathBuf::from("somewhere")));
        let decode = prepare_request(API_GATEWAY_REST_GET_SOMEWHERE_NOQUERY);
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/stage/somewhere");
        assert_eq!(
            req.inner().segments(0..),
            Ok(PathBuf::from("stage/somewhere"))
        );

        let decode = prepare_request(API_GATEWAY_V2_GET_SPACEPATH_NOQUERY);
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/path%20with/space");
        assert_eq!(
            req.inner().segments(0..),
            Ok(PathBuf::from("path with/space"))
        );
        let decode = prepare_request(API_GATEWAY_REST_GET_SPACEPATH_NOQUERY);
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/stage/path%20with/space");
        assert_eq!(
            req.inner().segments(0..),
            Ok(PathBuf::from("stage/path with/space"))
        );

        let decode = prepare_request(API_GATEWAY_V2_GET_PERCENTPATH_NOQUERY);
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/path%25with/percent");
        assert_eq!(
            req.inner().segments(0..),
            Ok(PathBuf::from("path%with/percent"))
        );
        let decode = prepare_request(API_GATEWAY_REST_GET_PERCENTPATH_NOQUERY);
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/stage/path%25with/percent");
        assert_eq!(
            req.inner().segments(0..),
            Ok(PathBuf::from("stage/path%with/percent"))
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
        let decode = prepare_request(API_GATEWAY_REST_GET_UTF8PATH_NOQUERY);
        let req = decode.make_request(&client);
        assert_eq!(
            &decode.path_and_query,
            "/stage/%E6%97%A5%E6%9C%AC%E8%AA%9E/%E3%83%95%E3%82%A1%E3%82%A4%E3%83%AB%E5%90%8D"
        );
        assert_eq!(
            req.inner().segments(0..),
            Ok(PathBuf::from("stage/日本語/ファイル名"))
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
        let decode = prepare_request(API_GATEWAY_REST_GET_ROOT_ONEQUERY);
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/stage/?key=value");
        assert_eq!(req.inner().segments(0..), Ok(PathBuf::from("stage")));
        assert_eq!(req.inner().query_value::<&str>("key").unwrap(), Ok("value"));

        let decode = prepare_request(API_GATEWAY_V2_GET_SOMEWHERE_ONEQUERY);
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/somewhere?key=value");
        assert_eq!(req.inner().segments(0..), Ok(PathBuf::from("somewhere")));
        assert_eq!(req.inner().query_value::<&str>("key").unwrap(), Ok("value"));
        let decode = prepare_request(API_GATEWAY_REST_GET_SOMEWHERE_ONEQUERY);
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/stage/somewhere?key=value");
        assert_eq!(
            req.inner().segments(0..),
            Ok(PathBuf::from("stage/somewhere"))
        );
        assert_eq!(req.inner().query_value::<&str>("key").unwrap(), Ok("value"));

        let decode = prepare_request(API_GATEWAY_V2_GET_SOMEWHERE_TWOQUERY);
        let req = decode.make_request(&client);
        assert_eq!(
            req.inner().query_value::<&str>("key1").unwrap(),
            Ok("value1")
        );
        assert_eq!(
            req.inner().query_value::<&str>("key2").unwrap(),
            Ok("value2")
        );
        let decode = prepare_request(API_GATEWAY_REST_GET_SOMEWHERE_TWOQUERY);
        let req = decode.make_request(&client);
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
        assert_eq!(
            req.inner().query_value::<&str>("key").unwrap(),
            Ok("value1 value2")
        );
        let decode = prepare_request(API_GATEWAY_REST_GET_SOMEWHERE_SPACEQUERY);
        let req = decode.make_request(&client);
        assert_eq!(
            req.inner().query_value::<&str>("key").unwrap(),
            Ok("value1 value2")
        );

        let decode = prepare_request(API_GATEWAY_V2_GET_SOMEWHERE_UTF8QUERY);
        let req = decode.make_request(&client);
        assert_eq!(
            req.inner().query_value::<&str>("key").unwrap(),
            Ok("日本語")
        );
        let decode = prepare_request(API_GATEWAY_REST_GET_SOMEWHERE_UTF8QUERY);
        let req = decode.make_request(&client);
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
        let decode = prepare_request(API_GATEWAY_REST_GET_ROOT_ONEQUERY);
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
        let decode = prepare_request(API_GATEWAY_REST_GET_REMOTE_IPV6);
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
        let decode = prepare_request(API_GATEWAY_REST_POST_FORM_URLENCODED);
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
        let decode = prepare_request(API_GATEWAY_REST_POST_FORM_URLENCODED_B64);
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
        let decode = prepare_request(API_GATEWAY_REST_GET_ROOT_NOQUERY);
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
        let decode = prepare_request(API_GATEWAY_REST_GET_ROOT_NOQUERY);
        let req = decode.make_request(&client);
        assert_eq!(req.inner().cookies().iter().count(), 0);

        let decode = prepare_request(API_GATEWAY_V2_GET_ONE_COOKIE);
        let req = decode.make_request(&client);
        assert_eq!(
            req.inner().cookies().get("cookie1").unwrap().value(),
            "value1"
        );
        let decode = prepare_request(API_GATEWAY_REST_GET_ONE_COOKIE);
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
        let decode = prepare_request(API_GATEWAY_REST_GET_TWO_COOKIES);
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
