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
    ApiGatewayHttpV2(ApiGatewayV2<'a>),
    ApiGatewayRestOrAlb(ApiGatewayRest<'a>),
}

impl LambdaHttpEvent<'_> {
    /// HTTP request method
    pub fn method<'a>(&'a self) -> &'a str {
        match self {
            Self::ApiGatewayHttpV2(event) => &event.request_context.http.method,
            Self::ApiGatewayRestOrAlb(event) => &event.http_method,
        }
    }

    /// URL encoded path?query
    pub fn path_query(&self) -> String {
        match self {
            Self::ApiGatewayHttpV2(event) => {
                let path = encode_path(&event.raw_path);
                let query = &event.raw_query_string as &str;
                if query.is_empty() {
                    // No query string
                    path.into_owned()
                } else {
                    // With query string
                    format!("{}?{}", path, query)
                }
            }
            Self::ApiGatewayRestOrAlb(event) => {
                let path = encode_path(&event.path);
                if event.multi_value_query_string_parameters.is_empty() {
                    // No query string
                    path.into_owned()
                } else {
                    // With query string
                    let querystr = event
                        .multi_value_query_string_parameters
                        .iter()
                        .flat_map(|(k, vec)| {
                            let k_enc = encode_query(&k);
                            vec.iter()
                                .map(move |v| format!("{}={}", k_enc, encode_query(&v)))
                        })
                        .collect::<Vec<_>>()
                        .join("&");
                    format!("{}?{}", path, querystr)
                }
            }
        }
    }

    /// Returns headers
    pub fn headers<'a>(&'a self) -> Vec<(&'a str, &'a str)> {
        match self {
            Self::ApiGatewayHttpV2(event) => event
                .headers
                .iter()
                .map(|(k, v)| (k as &str, v as &str))
                .collect(),
            Self::ApiGatewayRestOrAlb(event) => event
                .multi_value_headers
                .iter()
                .flat_map(|(k, vec)| vec.iter().map(move |v| (k as &str, v as &str)))
                .collect(),
        }
    }

    /// Cookies (percent-decoded)
    pub fn cookies<'a>(&'a self) -> Vec<cookie::Cookie<'a>> {
        match self {
            Self::ApiGatewayHttpV2(event) => {
                if let Some(cookies) = &event.cookies {
                    cookies
                        .iter()
                        .filter_map(|cookie_str| {
                            cookie::Cookie::parse_encoded(cookie_str as &str).ok()
                        })
                        .collect()
                } else {
                    // No cookie header
                    Vec::new()
                }
            }
            Self::ApiGatewayRestOrAlb(event) => {
                if let Some(cookies) = event.multi_value_headers.get("cookie") {
                    cookies
                        .iter()
                        .filter_map(|cookie_str| {
                            cookie::Cookie::parse_encoded(cookie_str as &str).ok()
                        })
                        .collect()
                } else {
                    // No cookie header
                    Vec::new()
                }
            }
        }
    }

    /// Single cookie header
    pub fn cookie_header_value(&self) -> Option<String> {
        let cookies = self.cookies();
        if cookies.is_empty() {
            None
        } else {
            let header_val = cookies
                .iter()
                .map(|c| c.encoded().stripped().to_string())
                .collect::<Vec<_>>()
                .join("; ");
            Some(header_val)
        }
    }

    /// Host name
    pub fn hostname<'a>(&'a self) -> Option<&'a str> {
        match self {
            Self::ApiGatewayHttpV2(event) => Some(&event.request_context.domain_name),
            Self::ApiGatewayRestOrAlb(event) => {
                if let Some(context) = &event.request_context {
                    Some(&context.domain_name)
                } else if let Some(host_headers) = event.multi_value_headers.get("host") {
                    host_headers.first().map(|h| h as &str)
                } else {
                    None
                }
            }
        }
    }

    ///
    pub fn client_supports_br(&self) -> bool {
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
}

/// API Gateway HTTP API payload format version 2.0
/// https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-develop-integrations-lambda.html
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApiGatewayV2<'a> {
    version: String,
    pub(crate) raw_path: String,
    pub(crate) raw_query_string: String,
    pub(crate) cookies: Option<Vec<String>>,
    pub(crate) headers: HashMap<String, String>,
    //#[serde(borrow)]
    pub(crate) body: Option<Cow<'a, str>>,
    #[serde(default)]
    pub(crate) is_base64_encoded: bool,
    pub(crate) request_context: ApiGatewayV2RequestContext,
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
pub(crate) struct ApiGatewayV2RequestContext {
    /// The full domain name used to invoke the API. This should be the same as the incoming Host header.
    pub(crate) domain_name: String,
    /// The HTTP method used.
    pub(crate) http: Http,
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
pub(crate) struct Http {
    /// The HTTP method used. Valid values include: DELETE, GET, HEAD, OPTIONS, PATCH, POST, and PUT.
    pub(crate) method: String,
    /// The source IP address of the TCP connection making the request to API Gateway.
    pub(crate) source_ip: String,
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
pub(crate) struct ApiGatewayRest<'a> {
    path: String,
    http_method: String,
    //#[serde(borrow)]
    body: Option<Cow<'a, str>>,
    #[serde(default)]
    is_base64_encoded: bool,
    multi_value_headers: HashMap<String, Vec<String>>,
    #[serde(default)]
    multi_value_query_string_parameters: HashMap<String, Vec<String>>,
    request_context: Option<ApiGatewayRestRequestContext>, // None when called from ALB
                                                           // headers: HashMap<String, String>,
                                                           // path_parameters: HashMap<String, String>,
                                                           // query_string_parameters: HashMap<String, String>,
                                                           // stage_variables: HashMap<String, String>,
}

/// API Gateway REST API request context
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ApiGatewayRestRequestContext {
    domain_name: String,
    // account_id: String,
    // api_id: String,
    // authorizer: HashMap<String, Value>,
    // domain_prefix: String,
    // http_method: String,
    // identity: HashMap<String, Value>,
    // path: String,
    // protocol: String,
    // request_id: String,
    // request_time: String,
    // request_time_epoch: i64,
    // resource_id: String,
    // resource_path: String,
    // stage: String,
}

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

fn encode_path<'a>(pathstr: &'a str) -> Cow<'a, str> {
    Cow::from(percent_encoding::utf8_percent_encode(
        pathstr,
        &RFC3986_PATH_ESCAPE_SET,
    ))
}

// multi_value_query_string_parameters in API Gateway REST API payload is percent decoded
// Query containing space or UTF-8 char is
// required to percent encoded again before passed to web frameworks
// See RFC3986 3.4 query for valid chars.
const RFC3986_QUERY_ESCAPE_SET: &percent_encoding::AsciiSet =
    &RFC3986_PATH_ESCAPE_SET.add(b'/').add(b'?');

fn encode_query<'a>(querystr: &'a str) -> Cow<'a, str> {
    Cow::from(percent_encoding::utf8_percent_encode(
        querystr,
        &RFC3986_QUERY_ESCAPE_SET,
    ))
}

impl<'a> ApiGatewayV2<'a> {
    pub(crate) fn encoded_path(&'a self) -> Cow<'a, str> {
        Cow::from(percent_encoding::utf8_percent_encode(
            &self.raw_path,
            &RFC3986_PATH_ESCAPE_SET,
        ))
    }
}
