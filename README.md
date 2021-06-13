# lambda-web

Run Actix web, ~~Rocket~~, ~~Warp~~ on AWS Lambda

## Features

### Web frameworks

#### Supported

* [Actix Web](https://crates.io/crates/actix-web/4.0.0-beta.6) 4.0.0-beta.6

#### Work in progress

* [Rocket](https://rocket.rs/)
* [Warp](https://github.com/seanmonstar/warp)

### AWS infrastructure

#### Supported

* [API Gateway HTTP API](https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api.html) with [payload format version 2.0](https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-develop-integrations-lambda.html#2.0)

#### Not supported

* API Gateway HTTP API with payload format version **1.0**
* API Gateway REST API
* Application Load Balancer (ALB)

## Example

### Actix Web

Cargo.toml

```toml
[dependencies]
lambda-web = { version = "0.1.0", features=["actix4"] }
```

main.rs

```rust
use lambda_web::actix_web::{self, get, App, HttpServer, Responder};
use lambda_web::{is_running_on_lambda, run_actix_on_lambda, LambdaError};


#[get("/")]
async fn hello() -> impl Responder {
    format!("Hello")
}

#[actix_web::main]
async fn main() -> Result<(),LambdaError> {
    let factory = move || {
        App::new().service(hello)
    };

    if is_running_on_lambda() {
        // Run on AWS Lambda
        run_actix_on_lambda(factory).await?;
    } else {
        // Local server
        HttpServer::new(factory)
            .bind("127.0.0.1:8080")?
            .run()
            .await?;
    }
}
```

## Create deploy ZIP file



## Setup AWS Lambda & API gateway

### Lambda

* Create lambda function with custom runtime. Choose "Provide your own bootstrap on Amazon Linux 2"
* Upload ZIP file described above.

### API Gateway

* Create HTTP API
* Create single route "$default" and attach Lambda integration. Make sure, payload format version is "2.0"
