// SPDX-License-Identifier: MIT
//!
//! Run Rocket on AWS Lambda
//!
//!
use crate::request::ApiGatewayV2;
use core::convert::TryFrom;
use core::future::Future;
use std::pin::Pin;
use std::sync::Arc;
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

impl LambdaHandler<ApiGatewayV2<'_>, serde_json::Value> for RocketHandler {
    type Error = rocket::Error;
    type Fut = Pin<Box<dyn Future<Output = Result<serde_json::Value, Self::Error>> + Send>>;

    /// Lambda handler function
    /// Parse Lambda event as Rocket LocalRequest,
    /// serialize Rocket LocalResponse to Lambda JSON response
    fn call(&mut self, event: ApiGatewayV2, _context: LambdaContext) -> Self::Fut {
        use serde_json::json;

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
                    api_gateway_response_from_rocket(response).await
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
    cookies: Vec<rocket::http::Cookie<'static>>,
    headers: Vec<rocket::http::Header<'static>>,
    body: Vec<u8>,
}

impl TryFrom<ApiGatewayV2<'_>> for RequestDecode {
    type Error = LambdaError;

    /// Request from API Gateway event
    fn try_from(event: ApiGatewayV2) -> Result<Self, Self::Error> {
        use rocket::http::{Cookie, Header, Method};
        use std::net::IpAddr;
        use std::str::FromStr;

        // path ? query_string
        let path_and_query = if event.raw_query_string.is_empty() {
            event.encoded_path().to_string()
        } else {
            format!("{}?{}", event.encoded_path(), event.raw_query_string)
        };

        // Method, Source IP
        let method = Method::from_str(&event.request_context.http.method as &str)
            .map_err(|_| "InvalidMethod")?;
        let source_ip = IpAddr::from_str(&event.request_context.http.source_ip as &str)?;

        // Parse cookies
        let cookies = if let Some(cookies) = event.cookies {
            cookies
                .iter()
                .filter_map(|cookie| {
                    Cookie::parse_encoded(cookie as &str)
                        .map(|c| c.into_owned())
                        .ok()
                })
                .collect::<Vec<Cookie>>()
        } else {
            vec![]
        };

        // Headers
        let headers = event
            .headers
            .iter()
            .map(|(k, v)| Header::new(k.to_string(), v.to_string()))
            .collect::<Vec<Header>>();

        // Body
        let body = if let Some(eventbody) = event.body {
            if event.is_base64_encoded {
                base64::decode(&eventbody as &str)?
            } else {
                Vec::<u8>::from(eventbody.into_owned())
            }
        } else {
            vec![]
        };

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
        // path, method, remote address, body
        let req = client
            .req(self.method, &self.path_and_query)
            .remote(std::net::SocketAddr::from((self.source_ip, 0u16)))
            .body(&self.body);

        // Copy cookies
        let req = self
            .cookies
            .iter()
            .fold(req, |req, cookie| req.cookie(cookie.clone()));

        // Copy headers
        let req = self
            .headers
            .iter()
            .fold(req, |req, header| req.header(header.clone()));

        req
    }
}

/// API Gateway response from Rocket response
async fn api_gateway_response_from_rocket(
    response: rocket::local::asynchronous::LocalResponse<'_>,
) -> Result<serde_json::Value, rocket::Error> {
    use serde_json::json;

    // HTTP status
    let status_code = response.status().code;

    // Convert header to JSON map
    let mut headers = serde_json::Map::new();
    for header in response.headers().iter() {
        headers.insert(header.name.into_string(), json!(header.value));
    }

    Ok(json!({
        "isBase64Encoded": true,
        "statusCode": status_code,
        "headers": headers,
        "body": base64::encode(response.into_bytes().await.unwrap_or_default())
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{request::ApiGatewayV2, test_consts::*};
    use rocket::{async_test, local::asynchronous::Client};
    use std::path::PathBuf;

    #[async_test]
    async fn test_path_decode() {
        let rocket = rocket::build();
        let client = Client::untracked(rocket).await.unwrap();

        let reqjson: ApiGatewayV2 = serde_json::from_str(API_GATEWAY_V2_GET_ROOT_NOQUERY).unwrap();
        let decode = RequestDecode::try_from(reqjson).unwrap();
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/");
        assert_eq!(req.inner().segments(0..), Ok(PathBuf::new()));

        let reqjson: ApiGatewayV2 =
            serde_json::from_str(API_GATEWAY_V2_GET_SOMEWHERE_NOQUERY).unwrap();
        let decode = RequestDecode::try_from(reqjson).unwrap();
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/somewhere");
        assert_eq!(req.inner().segments(0..), Ok(PathBuf::from("somewhere")));

        let reqjson: ApiGatewayV2 =
            serde_json::from_str(API_GATEWAY_V2_GET_SPACEPATH_NOQUERY).unwrap();
        let decode = RequestDecode::try_from(reqjson).unwrap();
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/path%20with/space");
        assert_eq!(req.inner().segments(0..), Ok(PathBuf::from("path with/space")));

        let reqjson: ApiGatewayV2 =
            serde_json::from_str(API_GATEWAY_V2_GET_PERCENTPATH_NOQUERY).unwrap();
        let decode = RequestDecode::try_from(reqjson).unwrap();
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/path%25with/percent");
        assert_eq!(req.inner().segments(0..), Ok(PathBuf::from("path%with/percent")));

        let reqjson: ApiGatewayV2 =
            serde_json::from_str(API_GATEWAY_V2_GET_UTF8PATH_NOQUERY).unwrap();
        let decode = RequestDecode::try_from(reqjson).unwrap();
        let req = decode.make_request(&client);
        assert_eq!(
            &decode.path_and_query,
            "/%E6%97%A5%E6%9C%AC%E8%AA%9E/%E3%83%95%E3%82%A1%E3%82%A4%E3%83%AB%E5%90%8D"
        );
        assert_eq!(req.inner().segments(0..), Ok(PathBuf::from("日本語/ファイル名")));
    }

    #[async_test]
    async fn test_query_decode() {
        let rocket = rocket::build();
        let client = Client::untracked(rocket).await.unwrap();

        let reqjson: ApiGatewayV2 =
            serde_json::from_str(API_GATEWAY_V2_GET_ROOT_ONEQUERY).unwrap();
        let decode = RequestDecode::try_from(reqjson).unwrap();
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/?key=value");
        assert_eq!(req.inner().segments(0..), Ok(PathBuf::new()));
        assert_eq!(req.inner().query_value::<&str>("key").unwrap(), Ok("value"));

        let reqjson: ApiGatewayV2 =
            serde_json::from_str(API_GATEWAY_V2_GET_SOMEWHERE_ONEQUERY).unwrap();
        let decode = RequestDecode::try_from(reqjson).unwrap();
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/somewhere?key=value");
        assert_eq!(req.inner().segments(0..), Ok(PathBuf::from("somewhere")));
        assert_eq!(req.inner().query_value::<&str>("key").unwrap(), Ok("value"));

        let reqjson: ApiGatewayV2 =
            serde_json::from_str(API_GATEWAY_V2_GET_SOMEWHERE_TWOQUERY).unwrap();
        let decode = RequestDecode::try_from(reqjson).unwrap();
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/somewhere?key1=value1&key2=value2");
        assert_eq!(req.inner().query_value::<&str>("key1").unwrap(), Ok("value1"));
        assert_eq!(req.inner().query_value::<&str>("key2").unwrap(), Ok("value2"));

        let reqjson: ApiGatewayV2 =
            serde_json::from_str(API_GATEWAY_V2_GET_SOMEWHERE_SPACEQUERY).unwrap();
        let decode = RequestDecode::try_from(reqjson).unwrap();
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/somewhere?key=value1+value2");
        assert_eq!(req.inner().query_value::<&str>("key").unwrap(), Ok("value1 value2"));

        let reqjson: ApiGatewayV2 =
            serde_json::from_str(API_GATEWAY_V2_GET_SOMEWHERE_UTF8QUERY).unwrap();
        let decode = RequestDecode::try_from(reqjson).unwrap();
        let req = decode.make_request(&client);
        assert_eq!(&decode.path_and_query, "/somewhere?key=%E6%97%A5%E6%9C%AC%E8%AA%9E");
        assert_eq!(req.inner().query_value::<&str>("key").unwrap(), Ok("日本語"));
    }
}
