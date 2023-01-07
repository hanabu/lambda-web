# lambda-web

Run Rust web server frameworks on AWS Lambda.
Currently, it supports Actix web, axum, Rocket, warp.

[![crates.io](https://img.shields.io/crates/v/lambda-web?label=latest)](https://crates.io/crates/lambda-web)
[![API docs](https://docs.rs/lambda-web/badge.svg)](https://docs.rs/lambda-web)

## Features

### Supported web frameworks

- [Actix Web](https://crates.io/crates/actix-web) 4.0
- [axum](https://crates.io/crates/axum) 0.6
- [Rocket](https://crates.io/crates/rocket/0.5.0-rc.2) 0.5.0-rc.2
- [warp](https://crates.io/crates/warp) 0.3

### Supported AWS infrastructure

- [API Gateway HTTP API](https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api.html) with [payload format version 2.0](https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-develop-integrations-lambda.html#2.0)
- [API Gateway REST API](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-rest-api.html)
- [Lambda function URLs](https://docs.aws.amazon.com/lambda/latest/dg/lambda-urls.html)

### Not supported

- API Gateway HTTP API with payload format version **1.0**
- [Application Load Balancer (ALB)](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/lambda-functions.html)

## Example

### Actix Web

Cargo.toml

```toml
[[bin]]
name = "bootstrap"
path = "src/main.rs"

[dependencies]
lambda-web = { version = "0.2.0", features=["actix4"] }
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
            .bind("127.0.0.2.0080")?
            .run()
            .await?;
    }
    Ok(())
}
```

### axum

Cargo.toml

```toml
[[bin]]
name = "bootstrap"
path = "src/main.rs"

[dependencies]
lambda-web = { version = "0.2.0", features=["hyper"] }
axum = "0.6"
tokio = { version = "1" }
```

main.rs

```rust
use axum::{routing::get, Router};
use lambda_web::{is_running_on_lambda, run_hyper_on_lambda, LambdaError};
use std::net::SocketAddr;

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    // build our application with a route
    let app = Router::new().route("/", get(root));

    if is_running_on_lambda() {
        // Run app on AWS Lambda
        run_hyper_on_lambda(app).await?;
    } else {
        // Run app on local server
        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    }
    Ok(())
}
```

### Rocket

Cargo.toml

```toml
[[bin]]
name = "bootstrap"
path = "src/main.rs"

[dependencies]
lambda-web = { version = "0.2.0", features=["rocket05"] }
rocket = "0.5.0-rc.2"
```

main.rs

```rust
use rocket::{self, get, routes};
use lambda_web::{is_running_on_lambda, launch_rocket_on_lambda, LambdaError};

#[get("/hello/<name>/<age>")]
fn hello(name: &str, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

#[rocket::main]
async fn main() -> Result<(), LambdaError> {
    let rocket = rocket::build().mount("/", routes![hello]);
    if is_running_on_lambda() {
        // Launch on AWS Lambda
        launch_rocket_on_lambda(rocket).await?;
    } else {
        // Launch local server
        let _ = rocket.launch().await?;
    }
    Ok(())
}
```

### warp

Cargo.toml

```toml
[[bin]]
name = "bootstrap"
path = "src/main.rs"

[dependencies]
lambda-web = { version = "0.2.0", features=["hyper"] }
warp = "0.3"
tokio = { version = "1" }
```

main.rs

```rust
use lambda_web::{is_running_on_lambda, run_hyper_on_lambda, LambdaError};
use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}", name));

    if is_running_on_lambda() {
        // Run on AWS Lambda
        run_hyper_on_lambda(warp::service(hello)).await?;
    } else {
        // Run local server
        warp::serve(hello).run(([127, 0, 0, 1], 8080)).await;
    }
    Ok(())
}
```

## Create deploy ZIP file

As of writing (Nov, 2021), we have two options to run Rust on AWS Lambda: [Amazon Linux 2 custom runtime](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-custom.html) or Docker container image.

I recommend ZIP deploy to Amazon Linux 2 custom runtime (`provided.al2`) because it's faster cold start time than container image.

To build Amazon Linux 2 compatible binary, see [Deploy.md](./Deploy.md) for more details.

## Setup AWS Lambda & function URLs

- Create lambda function with `provided.al2` custom runtime. Choose "Provide your own bootstrap on Amazon Linux 2" .
- Upload ZIP file described above.
- IAM role, memory settings, etc. are as your demands. \
  As sample code above consumes only 30MB of memory, many simple Rust app can fit in 128MB setting.
- Create function URL, then you can call your Lambda function with `https://<url-id>.lambda-url.<region>.on.aws`
- You can use CloudFront for custom domain.

## Setup AWS Lambda & API gateway

### Lambda

- Create lambda function with `provided.al2` custom runtime. Choose "Provide your own bootstrap on Amazon Linux 2" .
- Upload ZIP file described above.
- IAM role, memory settings, etc. are as your demands. \
  As sample code above consumes only 30MB of memory, many simple Rust app can fit in 128MB setting.

### API Gateway (HTTP)

- Create HTTP API
- Create single route "$default" and attach Lambda integration. Make sure, payload format version is "2.0"

### API Gateway (REST)

- Create REST API
- Create two resources:
  - ANY method on route `/` and attach Lambda proxy integration.
  - ANY method on route `/{proxy+}` and attach Lambda proxy integration.
- In settings tab, add `*/*` binary media type.
- Then, deploy API to stage.
