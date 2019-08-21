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

// TODO update doc comments - some functions now have different error/panic behaviour.

#[macro_use]
extern crate failure;

#[macro_use]
mod error;
mod handler;

use handler::LazyClient;
pub use handler::RocketHandler;
use lambda_http::lambda;
use rocket::Rocket;
use std::collections::HashMap;

/// Used to determine how to encode response content. The default is `Text`.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ResponseType {
    /// Send response content to API Gateway as a UTF-8 string.
    Text,
    /// Send response content to API Gateway Base64-encoded.
    Binary,
}

/// Extensions for `rocket::Rocket` to make it easier to create Lambda handlers.
pub trait RocketExt {
    /// Create a new `RocketLamb` from the given `Rocket`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket_lamb::RocketExt;
    ///
    /// let lamb = rocket::ignite().lambda();
    /// ```
    fn lambda(self) -> RocketLamb;
}

impl RocketExt for Rocket {
    fn lambda(self) -> RocketLamb {
        RocketLamb::new(self)
    }
}

/// A wrapper around a [rocket::Rocket] that can be used to handle Lambda events.
pub struct RocketLamb {
    rocket: Rocket,
    default_response_type: ResponseType,
    response_types: HashMap<String, ResponseType>,
    include_api_gateway_base_path: bool,
}

impl RocketLamb {
    /// Create a new `RocketLamb`. Alternatively, you can use [rocket.lambda()](RocketExt::lambda).
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket_lamb::RocketLamb;
    ///
    /// let lamb = RocketLamb::new(rocket::ignite());
    /// ```
    pub fn new(rocket: rocket::Rocket) -> RocketLamb {
        RocketLamb {
            rocket,
            default_response_type: ResponseType::Text,
            response_types: HashMap::new(),
            include_api_gateway_base_path: true,
        }
    }

    /// Creates a new `RocketHandler` from an instance of `Rocket`, which can be passed to the [lambda_http::lambda!](lambda_http::lambda) macro.
    ///
    /// Alternatively, you can use the [launch()](RocketLamb::launch) method.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rocket_lamb::RocketExt;
    /// use lambda_http::lambda;
    ///
    /// let handler = rocket::ignite().lambda().into_handler();
    /// lambda!(handler);
    /// ```
    pub fn into_handler(self) -> RocketHandler {
        RocketHandler {
            client: LazyClient::Uninitialized(self.rocket),
            default_response_type: self.default_response_type,
            response_types: self.response_types,
            include_api_gateway_base_path: self.include_api_gateway_base_path,
        }
    }

    /// Starts handling Lambda events.
    ///
    /// # Panics
    ///
    /// This panics if the required Lambda runtime environment variables are not set.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rocket_lamb::RocketExt;
    /// use lambda_http::lambda;
    ///
    /// rocket::ignite().lambda().launch();
    /// ```
    pub fn launch(self) -> ! {
        lambda!(self.into_handler());
        unreachable!("lambda! should loop forever (or panic)")
    }

    /// Gets the default `ResponseType`, which is used for any responses that have not had their Content-Type overriden with [response_type](RocketLamb::response_type).
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket_lamb::{RocketExt, ResponseType};
    ///
    /// let lamb = rocket::ignite().lambda();
    /// assert_eq!(lamb.get_default_response_type(), ResponseType::Text);
    /// assert_eq!(lamb.get_response_type("text/plain"), ResponseType::Text);
    /// ```
    pub fn get_default_response_type(&self) -> ResponseType {
        self.default_response_type
    }

    /// Sets the default `ResponseType`, which is used for any responses that have not had their Content-Type overriden with [response_type](RocketLamb::response_type).
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket_lamb::{RocketExt, ResponseType};
    ///
    /// let lamb = rocket::ignite()
    ///     .lambda()
    ///     .default_response_type(ResponseType::Binary);
    /// assert_eq!(lamb.get_default_response_type(), ResponseType::Binary);
    /// assert_eq!(lamb.get_response_type("text/plain"), ResponseType::Binary);
    /// ```
    pub fn default_response_type(mut self, response_type: ResponseType) -> Self {
        self.default_response_type = response_type;
        self
    }

    /// Gets the configured `ResponseType` for responses with the given Content-Type header.
    ///
    /// `content_type` values are treated case-insensitively.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket_lamb::{RocketExt, ResponseType};
    ///
    /// let lamb = rocket::ignite()
    ///     .lambda()
    ///     .response_type("TEXT/PLAIN", ResponseType::Binary);
    /// assert_eq!(lamb.get_response_type("text/plain"), ResponseType::Binary);
    /// assert_eq!(lamb.get_response_type("application/json"), ResponseType::Text);
    /// ```
    pub fn get_response_type(&self, content_type: &str) -> ResponseType {
        self.response_types
            .get(&content_type.to_lowercase())
            .copied()
            .unwrap_or(self.default_response_type)
    }

    /// Sets the `ResponseType` for responses with the given Content-Type header.
    ///
    /// `content_type` values are treated case-insensitively.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket_lamb::{RocketExt, ResponseType};
    ///
    /// let lamb = rocket::ignite()
    ///     .lambda()
    ///     .response_type("TEXT/PLAIN", ResponseType::Binary);
    /// assert_eq!(lamb.get_response_type("text/plain"), ResponseType::Binary);
    /// assert_eq!(lamb.get_response_type("application/json"), ResponseType::Text);
    /// ```
    pub fn response_type(mut self, content_type: &str, response_type: ResponseType) -> Self {
        self.response_types
            .insert(content_type.to_lowercase(), response_type);
        self
    }

    /// Sets whether or not the handler should determine the API Gateway base path and prepend it to the path of request URLs.
    ///
    /// When using the default API Gateway URL ({###}.execute-api.{region}.amazonaws.com/{stage}/), then the base path would
    /// be "/{stage}". If this setting is set to `true` (the default), then all mounted routes will be made available under
    /// "/{stage}", and all incoming requests to the Rocket webserver will have "/{stage}" at the beginning of the URL path.
    /// This is necessary to make absolute URLs in responses (e.g. in the `Location` response header for redirects) function
    /// correctly when hosting the server using the default API Gateway URL.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket_lamb::RocketExt;
    ///
    /// let lamb = rocket::ignite()
    ///     .lambda()
    ///     .include_api_gateway_base_path(false);
    /// ```
    pub fn include_api_gateway_base_path(mut self, setting: bool) -> Self {
        self.include_api_gateway_base_path = setting;
        self
    }
}
