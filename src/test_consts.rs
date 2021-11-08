//
// Path test
//

// GET /
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
pub(crate) const API_GATEWAY_REST_GET_ROOT_NOQUERY: &str = r###"{
    "path":"/",
    "httpMethod":"GET",
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "multiValueHeaders":{
        "x-forwarded-for":["1.2.3.4"],
        "x-forwarded-port":["443"],
        "x-forwarded-proto":["https"]
    },
    "queryStringParameters":{},
    "multiValueQueryStringParameters":{},
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "path":"/stage/",
        "identity":{
            "sourceIp": "1.2.3.4"
        }
    }
}"###;

// GET /somewhere
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
pub(crate) const API_GATEWAY_REST_GET_SOMEWHERE_NOQUERY: &str = r###"{
    "path":"/somewhere",
    "httpMethod":"GET",
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "multiValueHeaders":{
        "x-forwarded-for":["1.2.3.4"],
        "x-forwarded-port":["443"],
        "x-forwarded-proto":["https"]
    },
    "queryStringParameters":{},
    "multiValueQueryStringParameters":{},
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "path":"/stage/somewhere",
        "identity":{
            "sourceIp": "1.2.3.4"
        }
    }
}"###;

// GET /path%20with/space
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
pub(crate) const API_GATEWAY_REST_GET_SPACEPATH_NOQUERY: &str = r###"{
    "path":"/path%20with/space",
    "httpMethod":"GET",
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "multiValueHeaders":{
        "x-forwarded-for":["1.2.3.4"],
        "x-forwarded-port":["443"],
        "x-forwarded-proto":["https"]
    },
    "queryStringParameters":{},
    "multiValueQueryStringParameters":{},
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "path":"/stage/path%20with/space",        
        "identity":{
            "sourceIp": "1.2.3.4"
        }
    }
}"###;

// GET /path%25with/percent
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
pub(crate) const API_GATEWAY_REST_GET_PERCENTPATH_NOQUERY: &str = r###"{
    "path":"/path%25with/percent",
    "httpMethod":"GET",
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "multiValueHeaders":{
        "x-forwarded-for":["1.2.3.4"],
        "x-forwarded-port":["443"],
        "x-forwarded-proto":["https"]
    },
    "queryStringParameters":{},
    "multiValueQueryStringParameters":{},
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "path":"/stage/path%25with/percent",        
        "identity":{
            "sourceIp": "1.2.3.4"
        }
    }
}"###;

// GET /%E6%97%A5%E6%9C%AC%E8%AA%9E/%E3%83%95%E3%82%A1%E3%82%A4%E3%83%AB%E5%90%8D
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
pub(crate) const API_GATEWAY_REST_GET_UTF8PATH_NOQUERY: &str = r###"{
    "path":"/%E6%97%A5%E6%9C%AC%E8%AA%9E/%E3%83%95%E3%82%A1%E3%82%A4%E3%83%AB%E5%90%8D",
    "httpMethod":"GET",
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "multiValueHeaders":{
        "x-forwarded-for":["1.2.3.4"],
        "x-forwarded-port":["443"],
        "x-forwarded-proto":["https"]
    },
    "queryStringParameters":{},
    "multiValueQueryStringParameters":{},
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "path":"/stage/%E6%97%A5%E6%9C%AC%E8%AA%9E/%E3%83%95%E3%82%A1%E3%82%A4%E3%83%AB%E5%90%8D",        
        "identity":{
            "sourceIp": "1.2.3.4"
        }
    }
}"###;

//
// Query test
//

// GET /?key=value
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
pub(crate) const API_GATEWAY_REST_GET_ROOT_ONEQUERY: &str = r###"{
    "path":"/",
    "httpMethod":"GET",
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "multiValueHeaders":{
        "x-forwarded-for":["1.2.3.4"],
        "x-forwarded-port":["443"],
        "x-forwarded-proto":["https"]
    },
    "queryStringParameters":{
        "key":"value"
    },
    "multiValueQueryStringParameters":{
        "key":["value"]
    },
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "path":"/stage/",
        "identity":{
            "sourceIp": "1.2.3.4"
        }
    }
}"###;

// GET /somewhere?key=value
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
pub(crate) const API_GATEWAY_REST_GET_SOMEWHERE_ONEQUERY: &str = r###"{
    "path":"/somewhere",
    "httpMethod":"GET",
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "multiValueHeaders":{
        "x-forwarded-for":["1.2.3.4"],
        "x-forwarded-port":["443"],
        "x-forwarded-proto":["https"]
    },
    "queryStringParameters":{
        "key":"value"
    },
    "multiValueQueryStringParameters":{
        "key":["value"]
    },
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "path":"/stage/somewhere",
        "identity":{
            "sourceIp": "1.2.3.4"
        }
    }
}"###;

// GET /somewhere?key1=value1&key2=value2
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
pub(crate) const API_GATEWAY_REST_GET_SOMEWHERE_TWOQUERY: &str = r###"{
    "path":"/somewhere",
    "httpMethod":"GET",
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "multiValueHeaders":{
        "x-forwarded-for":["1.2.3.4"],
        "x-forwarded-port":["443"],
        "x-forwarded-proto":["https"]
    },
    "queryStringParameters":{
        "key1":"value1",
        "key2":"value2"
    },
    "multiValueQueryStringParameters":{
        "key1":["value1"],
        "key2":["value2"]
    },
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "path":"/stage/somewhere",
        "identity":{
            "sourceIp": "1.2.3.4"
        }
    }
}"###;

// GET /somewhere?key=value1+value2
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
pub(crate) const API_GATEWAY_REST_GET_SOMEWHERE_SPACEQUERY: &str = r###"{
    "path":"/somewhere",
    "httpMethod":"GET",
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "multiValueHeaders":{
        "x-forwarded-for":["1.2.3.4"],
        "x-forwarded-port":["443"],
        "x-forwarded-proto":["https"]
    },
    "queryStringParameters":{
        "key":"value1 value2"
    },
    "multiValueQueryStringParameters":{
        "key":["value1 value2"]
    },
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "path":"/stage/somewhere",
        "identity":{
            "sourceIp": "1.2.3.4"
        }
    }
}"###;

// GET /somewhere?key=%E6%97%A5%E6%9C%AC%E8%AA%9E
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
pub(crate) const API_GATEWAY_REST_GET_SOMEWHERE_UTF8QUERY: &str = r###"{
    "path":"/somewhere",
    "httpMethod":"GET",
    "headers":{
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "multiValueHeaders":{
        "x-forwarded-for":["1.2.3.4"],
        "x-forwarded-port":["443"],
        "x-forwarded-proto":["https"]
    },
    "queryStringParameters":{
        "key":"日本語"
    },
    "multiValueQueryStringParameters":{
        "key":["日本語"]
    },
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "path":"/stage/somewhere",
        "identity":{
            "sourceIp": "1.2.3.4"
        }
    }
}"###;

//
// IPv6 source IP
//
#[allow(dead_code)]
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
pub(crate) const API_GATEWAY_REST_GET_REMOTE_IPV6: &str = r###"{
    "path":"/",
    "httpMethod":"GET",
    "headers":{
        "x-forwarded-for":"2404:6800:400a:80c::2004",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "multiValueHeaders":{
        "x-forwarded-for":["2404:6800:400a:80c::2004"],
        "x-forwarded-port":["443"],
        "x-forwarded-proto":["https"]
    },
    "queryStringParameters":{},
    "multiValueQueryStringParameters":{},
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "path":"/stage/",
        "identity":{
            "sourceIp": "2404:6800:400a:80c::2004"
        }
    }
}"###;

//
// Cookie test
//

// Cookie: cookie1=value1
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
pub(crate) const API_GATEWAY_REST_GET_ONE_COOKIE: &str = r###"{
    "path":"/somewhere",
    "httpMethod":"GET",
    "headers":{
        "cookie":"cookie1=value1",
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "multiValueHeaders":{
        "cookie":["cookie1=value1"],
        "x-forwarded-for":["1.2.3.4"],
        "x-forwarded-port":["443"],
        "x-forwarded-proto":["https"]
    },
    "queryStringParameters":{},
    "multiValueQueryStringParameters":{},
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "path":"/stage/somewhere",
        "identity":{
            "sourceIp": "1.2.3.4"
        }
    }
}"###;

// Cookie: cookie1=value1; cookie2=value2
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
pub(crate) const API_GATEWAY_REST_GET_TWO_COOKIES: &str = r###"{
    "path":"/somewhere",
    "httpMethod":"GET",
    "headers":{
        "cookie":"cookie1=value1; cookie2=value2",
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "multiValueHeaders":{
        "cookie":["cookie1=value1; cookie2=value2"],
        "x-forwarded-for":["1.2.3.4"],
        "x-forwarded-port":["443"],
        "x-forwarded-proto":["https"]
    },
    "queryStringParameters":{},
    "multiValueQueryStringParameters":{},
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "path":"/stage/somewhere",
        "identity":{
            "sourceIp": "1.2.3.4"
        }
    }
}"###;

//
// Form POST
//

// POST /somewhere with key1=value1&key2=value2&Ok=Ok
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
pub(crate) const API_GATEWAY_REST_POST_FORM_URLENCODED: &str = r###"{
    "body":"key1=value1&key2=value2&Ok=Ok",
    "isBase64Encoded":false,
    "path":"/somewhere",
    "httpMethod":"POST",
    "headers":{
        "content-length":"29",
        "content-type":"application/x-www-form-urlencoded",
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "multiValueHeaders":{
        "content-length":["29"],
        "content-type":["application/x-www-form-urlencoded"],
        "x-forwarded-for":["1.2.3.4"],
        "x-forwarded-port":["443"],
        "x-forwarded-proto":["https"]
    },
    "queryStringParameters":{},
    "multiValueQueryStringParameters":{},
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "path":"/stage/somewhere",
        "identity":{
            "sourceIp": "1.2.3.4"
        }
    }
}"###;

// POST /somewhere with key1=value1&key2=value2&Ok=Ok, base64 encoded in API gateway
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
pub(crate) const API_GATEWAY_REST_POST_FORM_URLENCODED_B64: &str = r###"{
    "body":"a2V5MT12YWx1ZTEma2V5Mj12YWx1ZTImT2s9T2s=",
    "isBase64Encoded":true,
    "path":"/somewhere",
    "httpMethod":"POST",
    "headers":{
        "content-length":"29",
        "content-type":"application/x-www-form-urlencoded",
        "x-forwarded-for":"1.2.3.4",
        "x-forwarded-port":"443",
        "x-forwarded-proto":"https"
    },
    "multiValueHeaders":{
        "content-length":["29"],
        "content-type":["application/x-www-form-urlencoded"],
        "x-forwarded-for":["1.2.3.4"],
        "x-forwarded-port":["443"],
        "x-forwarded-proto":["https"]
    },
    "queryStringParameters":{},
    "multiValueQueryStringParameters":{},
    "requestContext":{
        "domainName":"yyyyyyyyyy.execute-api.ap-northeast-1.amazonaws.com",
        "path":"/stage/somewhere",
        "identity":{
            "sourceIp": "1.2.3.4"
        }
    }
}"###;
