# 🚀 Rocket Lamb 🐑

![Travis (.org)](https://img.shields.io/travis/GREsau/rocket-lamb?logo=travis) ![Crates.io](https://img.shields.io/crates/v/rocket_lamb)

A crate to allow running a [Rocket](https://rocket.rs/) webserver as an AWS Lambda Function with API Gateway, built on the [AWS Lambda Rust Runtime](https://github.com/awslabs/aws-lambda-rust-runtime).

The function takes a request from an AWS API Gateway Proxy and converts it into a `LocalRequest` to pass to Rocket. Then it will convert the response from Rocket into the response body that API Gateway understands.

This *should* also work with requests from an AWS Application Load Balancer, but this has not been tested.

## Usage

```rust
#![feature(proc_macro_hygiene, decl_macro)]

use rocket::routes;
use rocket_lamb::{lambda, RocketHandler};

fn main() {
    // ignite a new Rocket as you normally world, but instead of launching it...
    let rocket = rocket::ignite().mount("/", routes![/* ... */]);

    // ...use it to create a new RocketHandler:
    let handler = RocketHandler::new(rocket).unwrap();

    // then use this to fetch and handle Lambda events:
    lambda!(handler);
}
```
