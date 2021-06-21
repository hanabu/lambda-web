// SPDX-License-Identifier: MIT
pub use lamedh_runtime::Error as LambdaError;

#[cfg(test)]
#[cfg(any(feature = "actix4", feature = "rocket05", feature = "warp03"))]
mod test_consts;

#[cfg(any(feature = "actix4", feature = "rocket05", feature = "warp03"))]
pub(crate) mod brotli;
#[cfg(any(feature = "actix4", feature = "rocket05", feature = "warp03"))]
mod request;

#[cfg(feature = "actix4")]
mod actix4;
#[cfg(feature = "actix4")]
pub use actix4::run_actix_on_lambda;
#[cfg(feature = "actix4")]
pub use actix_web;

#[cfg(feature = "rocket05")]
mod rocket05;
#[cfg(feature = "rocket05")]
pub use rocket;
#[cfg(feature = "rocket05")]
pub use rocket05::launch_rocket_on_lambda;

#[cfg(feature = "warp03")]
mod warp03;
#[cfg(feature = "warp03")]
pub use warp;
#[cfg(feature = "warp03")]
pub use warp03::run_warp_on_lambda;

/// Returns true if it is running on AWS Lambda
pub fn is_running_on_lambda() -> bool {
    std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok()
}
