// SPDX-License-Identifier: MIT
//!
//! Lambda event deserialize
//!
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum LambdaHttpEvent<'a> {
    ApiGatewayHttpV2(ApiGatewayHttpV2Event<'a>),
    ApiGatewayRestOrAlb(ApiGatewayRestEvent<'a>),
}

impl LambdaHttpEvent<'_> {
    /// HTTP request method
    pub fn method<'a>(&'a self) -> &'a str {
        match self {
            Self::ApiGatewayHttpV2(event) => &event.request_context.http.method,
            Self::ApiGatewayRestOrAlb(event) => &event.http_method,
        }
    }

    /// Host name
    #[allow(dead_code)]
    pub fn hostname<'a>(&'a self) -> Option<&'a str> {
        match self {
            Self::ApiGatewayHttpV2(event) => Some(&event.request_context.domain_name),
            Self::ApiGatewayRestOrAlb(event) => {
                if let RestOrAlbRequestContext::Rest(context) = &event.request_context {
                    Some(&context.domain_name)
                } else if let Some(host_headers) = event.multi_value_headers.get("host") {
                    host_headers.first().map(|h| h as &str)
                } else {
                    None
                }
            }
        }
    }

    /// URL encoded path?query
    pub fn path_query(&self) -> String {
        match self {
            Self::ApiGatewayHttpV2(event) => {
                let path = encode_path_query(&event.raw_path);
                let query_string  = event.raw_query_string.clone().unwrap_or("".to_string());
                let query = query_string.as_str();

                if query.is_empty() {
                    // No query string
                    path.into_owned()
                } else {
                    // With query string
                    format!("{}?{}", path, query)
                }
            }
            Self::ApiGatewayRestOrAlb(event) => {
                let path = if let RestOrAlbRequestContext::Rest(context) = &event.request_context {
                    // API Gateway REST, request_contest.path contains stage prefix
                    &context.path
                } else {
                    // ALB
                    &event.path
                };
                if let Some(query_string_parameters) = &event.multi_value_query_string_parameters {
                    // With query string
                    let querystr = query_string_parameters
                        .iter()
                        .flat_map(|(k, vec)| {
                            let k_enc = encode_path_query(&k);
                            vec.iter()
                                .map(move |v| format!("{}={}", k_enc, encode_path_query(&v)))
                        })
                        .collect::<Vec<_>>()
                        .join("&");
                    format!("{}?{}", path, querystr)
                } else {
                    // No query string
                    path.clone()
                }
            }
        }
    }

    /// HTTP headers
    pub fn headers<'a>(&'a self) -> Vec<(&'a str, Cow<'a, str>)> {
        match self {
            Self::ApiGatewayHttpV2(event) => {
                let mut headers: Vec<(&'a str, Cow<'a, str>)> = event
                    .headers
                    .iter()
                    .map(|(k, v)| (k as &str, Cow::from(v as &str)))
                    .collect();

                // Add cookie header
                if let Some(cookies) = &event.cookies {
                    let cookie_value = cookies.join("; ");
                    headers.push(("cookie", Cow::from(cookie_value)));
                }

                headers
            }
            Self::ApiGatewayRestOrAlb(event) => event
                .multi_value_headers
                .iter()
                .flat_map(|(k, vec)| vec.iter().map(move |v| (k as &str, Cow::from(v as &str))))
                .collect(),
        }
    }

    /// Cookies
    /// percent encoded "key=val"
    #[allow(dead_code)]
    pub fn cookies<'a>(&'a self) -> Vec<&'a str> {
        match self {
            Self::ApiGatewayHttpV2(event) => {
                if let Some(cookies) = &event.cookies {
                    cookies.iter().map(|c| c.as_str()).collect()
                } else {
                    Vec::new()
                }
            }
            Self::ApiGatewayRestOrAlb(event) => {
                if let Some(cookie_headers) = event.multi_value_headers.get("cookie") {
                    cookie_headers
                        .iter()
                        .flat_map(|v| v.split(";"))
                        .map(|c| c.trim())
                        .collect()
                } else {
                    Vec::new()
                }
            }
        }
    }

    /// Check if HTTP client supports Brotli compression.
    /// ( Accept-Encoding contains "br" )
    #[cfg(feature = "br")]
    pub fn client_supports_brotli(&self) -> bool {
        match self {
            Self::ApiGatewayHttpV2(event) => {
                if let Some(header_val) = event.headers.get("accept-encoding") {
                    for elm in header_val.to_ascii_lowercase().split(',') {
                        if let Some(algo_name) = elm.split(';').next() {
                            // first part of elm, contains 'br', 'gzip', etc.
                            if algo_name.trim() == "br" {
                                // HTTP client support Brotli compression
                                return true;
                            }
                        }
                    }
                    // No "br" in accept-encoding header
                    false
                } else {
                    // No accept-encoding header
                    false
                }
            }
            Self::ApiGatewayRestOrAlb(event) => {
                if let Some(header_vals) = event.multi_value_headers.get("accept-encoding") {
                    for header_val in header_vals {
                        for elm in header_val.to_ascii_lowercase().split(',') {
                            if let Some(algo_name) = elm.split(';').next() {
                                // first part of elm, contains 'br', 'gzip', etc.
                                if algo_name.trim() == "br" {
                                    // HTTP client support Brotli compression
                                    return true;
                                }
                            }
                        }
                    }
                    // No "br" in accept-encoding header
                    false
                } else {
                    // No accept-encoding header
                    false
                }
            }
        }
    }

    // Without Brotli support, always returns false
    #[cfg(not(feature = "br"))]
    pub fn client_supports_brotli(&self) -> bool {
        false
    }

    /// Is request & response use multi-value-header
    pub fn multi_value(&self) -> bool {
        match self {
            Self::ApiGatewayHttpV2(_) => false,
            Self::ApiGatewayRestOrAlb(_) => true,
        }
    }

    /// Request body
    pub fn body(self) -> Result<Vec<u8>, base64::DecodeError> {
        let (body, b64_encoded) = match self {
            Self::ApiGatewayHttpV2(event) => (event.body, event.is_base64_encoded),
            Self::ApiGatewayRestOrAlb(event) => (event.body, event.is_base64_encoded),
        };

        if let Some(body) = body {
            if b64_encoded {
                // base64 decode
                base64::decode(&body as &str)
            } else {
                // string
                Ok(body.into_owned().into_bytes())
            }
        } else {
            // empty body (GET, OPTION, etc. methods)
            Ok(Vec::new())
        }
    }

    /// Source IP address
    #[allow(dead_code)]
    pub fn source_ip(&self) -> Option<std::net::IpAddr> {
        use std::net::IpAddr;
        use std::str::FromStr;
        match self {
            Self::ApiGatewayHttpV2(event) => {
                IpAddr::from_str(&event.request_context.http.source_ip).ok()
            }
            Self::ApiGatewayRestOrAlb(event) => {
                if let RestOrAlbRequestContext::Rest(context) = &event.request_context {
                    IpAddr::from_str(&context.identity.source_ip).ok()
                } else {
                    None
                }
            }
        }
    }
}

/// API Gateway HTTP API payload format version 2.0
/// https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-develop-integrations-lambda.html
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApiGatewayHttpV2Event<'a> {
    #[allow(dead_code)]
    version: String,
    raw_path: String,
    raw_query_string: Option<String>,
    cookies: Option<Vec<String>>,
    headers: HashMap<String, String>,
    //#[serde(borrow)]
    body: Option<Cow<'a, str>>,
    #[serde(default)]
    is_base64_encoded: bool,
    request_context: ApiGatewayV2RequestContext,
    // route_key: Cow<'a, str>,
    // #[serde(default)]
    // query_string_parameters: StrMap,
    // #[serde(default)]
    // path_parameters: StrMap,
    // #[serde(default)]
    // stage_variables: StrMap,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ApiGatewayV2RequestContext {
    /// The full domain name used to invoke the API. This should be the same as the incoming Host header.
    domain_name: String,
    /// The HTTP method used.
    http: Http,
    // The API owner's AWS account ID.
    // pub account_id: String,
    // The identifier API Gateway assigns to your API.
    // pub api_id: String,
    // The stringified value of the specified key-value pair of the context map returned from an API Gateway Lambda authorizer function.
    // #[serde(default)]
    // pub authorizer: HashMap<String, serde_json::Value>,
    // The first label of the $context.domainName. This is often used as a caller/customer identifier.
    // pub domain_prefix: String,
    // The ID that API Gateway assigns to the API request.
    // pub request_id: String,
    // Undocumented, could be resourcePath
    // pub route_key: String,
    // The deployment stage of the API request (for example, Beta or Prod).
    // pub stage: String,
    // Undocumented, could be requestTime
    // pub time: String,
    // Undocumented, could be requestTimeEpoch
    // pub time_epoch: usize,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct Http {
    /// The HTTP method used. Valid values include: DELETE, GET, HEAD, OPTIONS, PATCH, POST, and PUT.
    method: String,
    /// The source IP address of the TCP connection making the request to API Gateway.
    source_ip: String,
    // The request path. For example, for a non-proxy request URL of
    // `https://{rest-api-id.execute-api.{region}.amazonaws.com/{stage}/root/child`,
    // the $context.path value is `/{stage}/root/child`.
    // pub path: String,
    // The request protocol, for example, HTTP/1.1.
    // pub protocol: String,
    // The User-Agent header of the API caller.
    // pub user_agent: String,
}

/// API Gateway REST API, ALB payload format
/// https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html#api-gateway-simple-proxy-for-lambda-input-format
///
/// In case of ALB, you must explicitly enable multi-value headers setting.
///
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApiGatewayRestEvent<'a> {
    // path without stage
    path: String,
    http_method: String,
    //#[serde(borrow)]
    body: Option<Cow<'a, str>>,
    #[serde(default)]
    is_base64_encoded: bool,
    multi_value_headers: HashMap<String, Vec<String>>,
    #[serde(default)]
    multi_value_query_string_parameters: Option<HashMap<String, Vec<String>>>,
    // request_context = None when called from ALB
    request_context: RestOrAlbRequestContext,
    // headers: HashMap<String, String>,
    // path_parameters: HashMap<String, String>,
    // query_string_parameters: HashMap<String, String>,
    // stage_variables: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum RestOrAlbRequestContext {
    Rest(ApiGatewayRestRequestContext),
    Alb(AlbRequestContext),
}

/// API Gateway REST API request context
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ApiGatewayRestRequestContext {
    domain_name: String,
    identity: ApiGatewayRestIdentity,
    // Path with stage
    path: String,
    // account_id: String,
    // api_id: String,
    // authorizer: HashMap<String, Value>,
    // domain_prefix: String,
    // http_method: String,
    // protocol: String,
    // request_id: String,
    // request_time: String,
    // request_time_epoch: i64,
    // resource_id: String,
    // resource_path: String,
    // stage: String,
}

/// API Gateway REST API identity
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ApiGatewayRestIdentity {
    #[allow(dead_code)]
    access_key: Option<String>,
    source_ip: String,
}

/// ALB Request context
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AlbRequestContext {}

// raw_path in API Gateway HTTP API V2 payload is percent decoded.
// Path containing space or UTF-8 char is
// required to percent encoded again before passed to web frameworks
// See RFC3986 3.3 Path for valid chars.
const RFC3986_PATH_ESCAPE_SET: &percent_encoding::AsciiSet = &percent_encoding::CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'+')
    .add(b':')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

fn encode_path_query<'a>(pathstr: &'a str) -> Cow<'a, str> {
    Cow::from(percent_encoding::utf8_percent_encode(
        pathstr,
        &RFC3986_PATH_ESCAPE_SET,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_consts::*;

    #[test]
    fn test_decode() {
        let _: ApiGatewayHttpV2Event =
            serde_json::from_str(API_GATEWAY_V2_GET_ROOT_NOQUERY).unwrap();
        let _: LambdaHttpEvent = serde_json::from_str(API_GATEWAY_V2_GET_ROOT_NOQUERY).unwrap();
        let _: ApiGatewayRestEvent =
            serde_json::from_str(API_GATEWAY_REST_GET_ROOT_NOQUERY).unwrap();
        let _: LambdaHttpEvent = serde_json::from_str(API_GATEWAY_REST_GET_ROOT_NOQUERY).unwrap();
    }

    #[test]
    fn test_cookie() {
        let event: LambdaHttpEvent = serde_json::from_str(API_GATEWAY_V2_GET_TWO_COOKIES).unwrap();
        assert_eq!(
            event.cookies(),
            vec!["cookie1=value1".to_string(), "cookie2=value2".to_string()]
        );
        let event: LambdaHttpEvent =
            serde_json::from_str(API_GATEWAY_REST_GET_TWO_COOKIES).unwrap();
        assert_eq!(
            event.cookies(),
            vec!["cookie1=value1".to_string(), "cookie2=value2".to_string()]
        );
    }
}
