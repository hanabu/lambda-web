use lambda_web::warp::{self, Filter};
use lambda_web::{is_running_on_lambda, run_warp_on_lambda, LambdaError};

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    if is_running_on_lambda() {
        // Run on AWS Lambda
        run_warp_on_lambda(warp::service(hello)).await?;
    } else {
        // Run local server
        warp::serve(hello).run(([127, 0, 0, 1], 8080)).await;
    }
    Ok(())
}
