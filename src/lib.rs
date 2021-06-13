mod request;
pub use lamedh_runtime::Error as LambdaError;

#[cfg(feature = "actix4")]
mod actix4;
#[cfg(feature = "actix4")]
pub use actix4::run_actix_on_lambda;
#[cfg(feature = "actix4")]
pub use actix_web;

/// Returns true if it is running on AWS Lambda
pub fn is_running_on_lambda() -> bool {
    std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok()
}
