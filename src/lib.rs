/*!
A crate to allow running a [Rocket](https://rocket.rs/) webserver as an AWS Lambda Function with API Gateway, built on the [AWS Lambda Rust Runtime](https://github.com/awslabs/aws-lambda-rust-runtime).

The function takes a request from an AWS API Gateway Proxy and converts it into a `LocalRequest` to pass to Rocket. Then it will convert the response from Rocket into the response body that API Gateway understands.

This *should* also work with requests from an AWS Application Load Balancer, but this has not been tested.

## Usage

```rust,no_run
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
*/
#[macro_use]
extern crate failure;

#[macro_use]
mod error;

use error::RocketLambError;
use lambda_http::{Body, Handler, Request, RequestExt, Response};
use lambda_runtime::{error::HandlerError, Context};
use rocket::error::LaunchError;
use rocket::http::{uri::Uri, Header};
use rocket::local::{Client, LocalRequest, LocalResponse};
use std::collections::HashMap;

pub use lambda_http::lambda;

/// Used to determine how to encode response content. The default is `Text`.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ResponseType {
    /// Send response content to API Gateway as a UTF-8 string.
    Text,
    /// Send response content to API Gateway Base64-encoded.
    Binary,
}

/// A Lambda handler for API Gateway events that processes requests using `Rocket`.
pub struct RocketHandler {
    client: Client,
    default_response_type: ResponseType,
    response_types: HashMap<String, ResponseType>,
}

impl Handler<Response<Body>> for RocketHandler {
    fn run(&mut self, req: Request, _ctx: Context) -> Result<Response<Body>, HandlerError> {
        self.run_internal(req)
            .map_err(failure::Error::from)
            .map_err(failure::Error::into)
    }
}

impl RocketHandler {
    /// Creates a new `RocketHandler` from an instance of `Rocket`.
    ///
    /// # Errors
    ///
    /// If launching the `Rocket` instance would fail, excepting network errors, the `LaunchError` is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket_lamb::RocketHandler;
    ///
    /// let handler = RocketHandler::new(rocket::ignite())?;
    /// # Ok::<(), rocket::error::LaunchError>(())
    /// ```
    pub fn new(rocket: rocket::Rocket) -> Result<RocketHandler, LaunchError> {
        let client = Client::untracked(rocket)?;
        Ok(RocketHandler {
            client,
            default_response_type: ResponseType::Text,
            response_types: HashMap::new(),
        })
    }

    /// Gets the default [ResponseType], which is used for any responses that have not had their Content-Type overriden with [response_type](RocketHandler::response_type).
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket_lamb::{RocketHandler, ResponseType};
    ///
    /// let handler = RocketHandler::new(rocket::ignite())?;
    /// assert_eq!(handler.get_default_response_type(), ResponseType::Text);
    /// assert_eq!(handler.get_response_type("text/plain"), ResponseType::Text);
    /// # Ok::<(), rocket::error::LaunchError>(())
    /// ```
    pub fn get_default_response_type(&self) -> ResponseType {
        self.default_response_type
    }

    /// Sets the default [ResponseType], which is used for any responses that have not had their Content-Type overriden with [response_type](RocketHandler::response_type).
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket_lamb::{RocketHandler, ResponseType};
    ///
    /// let handler = RocketHandler::new(rocket::ignite())?
    ///     .default_response_type(ResponseType::Binary);
    /// assert_eq!(handler.get_default_response_type(), ResponseType::Binary);
    /// assert_eq!(handler.get_response_type("text/plain"), ResponseType::Binary);
    /// # Ok::<(), rocket::error::LaunchError>(())
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
    /// use rocket_lamb::{RocketHandler, ResponseType};
    ///
    /// let handler = RocketHandler::new(rocket::ignite())?
    ///     .response_type("TEXT/PLAIN", ResponseType::Binary);
    /// assert_eq!(handler.get_response_type("text/plain"), ResponseType::Binary);
    /// assert_eq!(handler.get_response_type("application/json"), ResponseType::Text);
    /// # Ok::<(), rocket::error::LaunchError>(())
    /// ```
    pub fn get_response_type(&self, content_type: &str) -> ResponseType {
        self.response_types
            .get(&content_type.to_lowercase())
            .map(|rt| *rt)
            .unwrap_or(self.get_default_response_type())
    }

    /// Sets the `ResponseType` for responses with the given Content-Type header.
    ///
    /// `content_type` values are treated case-insensitively.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket_lamb::{RocketHandler, ResponseType};
    ///
    /// let handler = RocketHandler::new(rocket::ignite())?
    ///     .response_type("TEXT/PLAIN", ResponseType::Binary);
    /// assert_eq!(handler.get_response_type("text/plain"), ResponseType::Binary);
    /// # Ok::<(), rocket::error::LaunchError>(())
    /// ```
    pub fn response_type(mut self, content_type: &str, response_type: ResponseType) -> Self {
        self.response_types
            .insert(content_type.to_lowercase(), response_type);
        self
    }

    fn run_internal(&self, req: Request) -> Result<Response<Body>, RocketLambError> {
        let local_req = self.create_rocket_request(req)?;
        let local_res = local_req.dispatch();
        self.create_lambda_response(local_res)
    }

    fn create_rocket_request(&self, req: Request) -> Result<LocalRequest, RocketLambError> {
        let method = to_rocket_method(req.method())?;
        let uri = get_path_and_query(&req);
        let mut local_req = self.client.req(method, uri);
        for (name, value) in req.headers() {
            match value.to_str() {
                Ok(v) => local_req.add_header(Header::new(name.to_string(), v.to_string())),
                Err(_) => return Err(invalid_request!("invalid value for header '{}'", name)),
            }
        }
        local_req.set_body(req.into_body());
        Ok(local_req)
    }

    fn create_lambda_response(
        &self,
        mut local_res: LocalResponse,
    ) -> Result<Response<Body>, RocketLambError> {
        let mut builder = Response::builder();
        builder.status(local_res.status().code);
        for h in local_res.headers().iter() {
            builder.header(&h.name.to_string(), &h.value.to_string());
        }

        let response_type = local_res
            .headers()
            .get_one("content-type")
            .unwrap_or_default()
            .split(';')
            .next()
            .map(|ct| self.get_response_type(ct))
            .unwrap_or(self.default_response_type);
        let body = match (local_res.body(), response_type) {
            (Some(b), ResponseType::Text) => Body::Text(
                b.into_string()
                    .ok_or_else(|| invalid_response!("response body was not text"))?,
            ),
            (Some(b), ResponseType::Binary) => Body::Binary(b.into_bytes().unwrap_or_default()),
            (None, _) => Body::Empty,
        };

        builder.body(body).map_err(|e| invalid_response!("{}", e))
    }
}

fn get_path_and_query(req: &Request) -> String {
    let mut uri = req.uri().path().to_string();
    let query = req.query_string_parameters();

    let mut separator = '?';
    for (key, _) in query.iter() {
        for value in query.get_all(key).unwrap() {
            uri.push_str(&format!(
                "{}{}={}",
                separator,
                Uri::percent_encode(key),
                Uri::percent_encode(value)
            ));
            separator = '&';
        }
    }
    uri
}

fn to_rocket_method(method: &http::Method) -> Result<rocket::http::Method, RocketLambError> {
    use http::Method as H;
    use rocket::http::Method::*;
    Ok(match *method {
        H::GET => Get,
        H::PUT => Put,
        H::POST => Post,
        H::DELETE => Delete,
        H::OPTIONS => Options,
        H::HEAD => Head,
        H::TRACE => Trace,
        H::CONNECT => Connect,
        H::PATCH => Patch,
        _ => return Err(invalid_request!("unknown method '{}'", method)),
    })
}
