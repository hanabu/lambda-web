use lambda_web::{is_running_on_lambda, launch_rocket_on_lambda, LambdaError};
use rocket::{self, get, routes};

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
        rocket.launch().await?;
    }
    Ok(())
}
