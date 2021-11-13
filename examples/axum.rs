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
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root));

    if is_running_on_lambda() {
        // Run app on AWS Lambda
        run_hyper_on_lambda(app).await?;
    } else {
        // Run app on local server
        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await?;
    }
    Ok(())
}
