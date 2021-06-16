//
// Path test
//
pub(crate) const API_GATEWAY_V2_GET_ROOT_NOQUERY: &str = r###"{
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "isBase64Encoded":false,
    "rawPath":"/",
    "rawQueryString":"",
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "http":{
            "method":"GET",
            "sourceIp":"1.2.3.4"
        }
    },
    "version":"2.0"
}"###;

pub(crate) const API_GATEWAY_V2_GET_SOMEWHERE_NOQUERY: &str = r###"{
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "isBase64Encoded":false,
    "rawPath":"/somewhere",
    "rawQueryString":"",
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "http":{
            "method":"GET",
            "sourceIp":"1.2.3.4"
        }
    },
    "version":"2.0"
}"###;

pub(crate) const API_GATEWAY_V2_GET_SPACEPATH_NOQUERY: &str = r###"{
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "isBase64Encoded":false,
    "rawPath":"/path with/space",
    "rawQueryString":"",
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "http":{
            "method":"GET",
            "sourceIp":"1.2.3.4"
        }
    },
    "version":"2.0"
}"###;

pub(crate) const API_GATEWAY_V2_GET_PERCENTPATH_NOQUERY: &str = r###"{
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "isBase64Encoded":false,
    "rawPath":"/path%with/percent",
    "rawQueryString":"",
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "http":{
            "method":"GET",
            "sourceIp":"1.2.3.4"
        }
    },
    "version":"2.0"
}"###;

pub(crate) const API_GATEWAY_V2_GET_UTF8PATH_NOQUERY: &str = r###"{
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "isBase64Encoded":false,
    "rawPath":"/日本語/ファイル名",
    "rawQueryString":"",
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "http":{
            "method":"GET",
            "sourceIp":"1.2.3.4"
        }
    },
    "version":"2.0"
}"###;

//
// Query test
//
pub(crate) const API_GATEWAY_V2_GET_ROOT_ONEQUERY: &str = r###"{
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "isBase64Encoded":false,
    "rawPath":"/",
    "rawQueryString":"key=value",
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "http":{
            "method":"GET",
            "sourceIp":"1.2.3.4"
        }
    },
    "version":"2.0"
}"###;

pub(crate) const API_GATEWAY_V2_GET_SOMEWHERE_ONEQUERY: &str = r###"{
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "isBase64Encoded":false,
    "rawPath":"/somewhere",
    "rawQueryString":"key=value",
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "http":{
            "method":"GET",
            "sourceIp":"1.2.3.4"
        }
    },
    "version":"2.0"
}"###;

pub(crate) const API_GATEWAY_V2_GET_SOMEWHERE_TWOQUERY: &str = r###"{
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "isBase64Encoded":false,
    "rawPath":"/somewhere",
    "rawQueryString":"key1=value1&key2=value2",
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "http":{
            "method":"GET",
            "sourceIp":"1.2.3.4"
        }
    },
    "version":"2.0"
}"###;

pub(crate) const API_GATEWAY_V2_GET_SOMEWHERE_SPACEQUERY: &str = r###"{
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "isBase64Encoded":false,
    "rawPath":"/somewhere",
    "rawQueryString":"key=value1+value2",
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "http":{
            "method":"GET",
            "sourceIp":"1.2.3.4"
        }
    },
    "version":"2.0"
}"###;

pub(crate) const API_GATEWAY_V2_GET_SOMEWHERE_UTF8QUERY: &str = r###"{
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "isBase64Encoded":false,
    "rawPath":"/somewhere",
    "rawQueryString":"key=%E6%97%A5%E6%9C%AC%E8%AA%9E",
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "http":{
            "method":"GET",
            "sourceIp":"1.2.3.4"
        }
    },
    "version":"2.0"
}"###;

//
// IPv6 source IP
//
pub(crate) const API_GATEWAY_V2_GET_REMOTE_IPV6: &str = r###"{
    "headers":{
        "x-forwarded-for":"2404:6800:400a:80c::2004",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "isBase64Encoded":false,
    "rawPath":"/",
    "rawQueryString":"",
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "http":{
            "method":"GET",
            "sourceIp":"2404:6800:400a:80c::2004"
        }
    },
    "version":"2.0"
}"###;

//
// Cookie test
//
pub(crate) const API_GATEWAY_V2_GET_ONE_COOKIE: &str = r###"{
    "cookies":["cookie1=value1"],
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "isBase64Encoded":false,
    "rawPath":"/",
    "rawQueryString":"",
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "http":{
            "method":"GET",
            "sourceIp":"1.2.3.4"
        }
    },
    "version":"2.0"
}"###;

pub(crate) const API_GATEWAY_V2_GET_TWO_COOKIES: &str = r###"{
    "cookies":["cookie2=value2","cookie1=value1"],
        "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "isBase64Encoded":false,
    "rawPath":"/",
    "rawQueryString":"",
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "http":{
            "method":"GET",
            "sourceIp":"1.2.3.4"
        }
    },
    "version":"2.0"
}"###;


//
// Form POST
//
pub(crate) const API_GATEWAY_V2_POST_FORM_URLENCODED: &str = r###"{
    "body":"key1=value1&key2=value2&Ok=Ok",
    "headers":{
        "content-length":"29",
        "content-type":"application/x-www-form-urlencoded",
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "isBase64Encoded":false,
    "rawPath":"/somewhere",
    "rawQueryString":"",
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "http":{
            "method":"POST",
            "sourceIp":"1.2.3.4"
        }
    },
    "version":"2.0"
}"###;

pub(crate) const API_GATEWAY_V2_POST_FORM_URLENCODED_B64: &str = r###"{
    "body":"a2V5MT12YWx1ZTEma2V5Mj12YWx1ZTImT2s9T2s=",
    "headers":{
        "content-length":"29",
        "content-type":"application/x-www-form-urlencoded",
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "isBase64Encoded":true,
    "rawPath":"/somewhere",
    "rawQueryString":"",
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "http":{
            "method":"POST",
            "sourceIp":"1.2.3.4"
        }
    },
    "version":"2.0"
}"###;

