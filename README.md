# 🚀 Rocket Lamb 🐑

[![Travis (.org)](https://img.shields.io/travis/GREsau/rocket-lamb?logo=travis)](https://travis-ci.org/GREsau/rocket-lamb)
[![Crates.io](https://img.shields.io/crates/v/rocket_lamb)](https://crates.io/crates/rocket_lamb)

A crate to allow running a [Rocket](https://rocket.rs/) webserver as an AWS Lambda Function with API Gateway, built on the [AWS Lambda Rust Runtime](https://github.com/awslabs/aws-lambda-rust-runtime).

The function takes a request from an AWS API Gateway Proxy and converts it into a `LocalRequest` to pass to Rocket. Then it will convert the response from Rocket into the response body that API Gateway understands.

This *should* also work with requests from an AWS Application Load Balancer, but this has not been tested.

## Usage

```rust
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

For a full example including instructions on deploying to Lambda and configuring binary responses, see [Example Rocket Lamb API](https://github.com/GREsau/example-rocket-lamb-api).
