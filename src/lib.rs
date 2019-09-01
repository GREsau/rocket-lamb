/*!
A crate to allow running a [Rocket](https://rocket.rs/) webserver as an AWS Lambda Function with API Gateway, built on the [AWS Lambda Rust Runtime](https://github.com/awslabs/aws-lambda-rust-runtime).

The function takes a request from an AWS API Gateway Proxy and converts it into a `LocalRequest` to pass to Rocket. Then it will convert the response from Rocket into the response body that API Gateway understands.

This *should* also work with requests from an AWS Application Load Balancer, but this has not been tested.

## Usage

```rust,no_run
#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
use rocket_lamb::RocketExt;

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

fn main() {
    rocket::ignite()
        .mount("/hello", routes![hello])
        .lambda() // launch the Rocket as a Lambda
        .launch();
}
```
*/

#![allow(clippy::large_enum_variant)]

use rocket::Rocket;

#[macro_use]
extern crate failure;

#[macro_use]
mod error;

mod builder;
mod config;
mod handler;
mod request_ext;

pub use builder::*;
pub use config::*;
pub use handler::*;

/// Extensions for `rocket::Rocket` to make it easier to create Lambda handlers.
pub trait RocketExt {
    /// Create a new `RocketHandlerBuilder` from the given `Rocket`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket_lamb::RocketExt;
    ///
    /// let builder = rocket::ignite().lambda();
    /// ```
    fn lambda(self) -> RocketHandlerBuilder;
}

impl RocketExt for Rocket {
    fn lambda(self) -> RocketHandlerBuilder {
        RocketHandlerBuilder::new(self)
    }
}
