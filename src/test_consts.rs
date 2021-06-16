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
