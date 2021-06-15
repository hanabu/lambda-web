use lambda_web::actix_web::{self, get, App, HttpServer, Responder};
use lambda_web::{is_running_on_lambda, run_actix_on_lambda, LambdaError};

#[get("/")]
async fn hello() -> impl Responder {
    format!("Hello")
}

#[actix_web::main]
async fn main() -> Result<(), LambdaError> {
    let factory = move || App::new().service(hello);
    if is_running_on_lambda() {
        // Run on AWS Lambda
        run_actix_on_lambda(factory).await?;
    } else {
        // Run local server
        HttpServer::new(factory)
            .bind("127.0.0.1:8080")?
            .run()
            .await?;
    }
    Ok(())
}
