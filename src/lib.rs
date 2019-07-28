/*!
A crate to allow running a [Rocket](https://rocket.rs/) webserver as an AWS Lambda Function with API Gateway, built on the [AWS Lambda Rust Runtime](https://github.com/awslabs/aws-lambda-rust-runtime).

The function takes a request from an AWS API Gateway Proxy and converts it into a `LocalRequest` to pass to Rocket. Then it will convert the response from Rocket into the response body that API Gateway understands.

This *should* also work with requests from an AWS Application Load Balancer, but this has not been tested.

## Installation

Add the following to your Cargo.toml `[dependencies]`:

```toml
rocket_lamb = "0.1.0"
```

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
use rocket::http::{uri::Uri, Header, Method};
use rocket::local::{Client, LocalRequest, LocalResponse};

pub use lambda_http::lambda;

/// A Lambda handler for API Gateway events that processes requests using `Rocket`.
pub struct RocketHandler(Client);

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
    /// # use rocket_lamb::RocketHandler;
    /// let handler = RocketHandler::new(rocket::ignite())?;
    /// # Ok::<(), rocket::error::LaunchError>(())
    /// ```
    pub fn new(rocket: rocket::Rocket) -> Result<RocketHandler, LaunchError> {
        let client = Client::untracked(rocket)?;
        Ok(RocketHandler(client))
    }
}

impl Handler<Response<Body>> for RocketHandler {
    fn run(&mut self, req: Request, _ctx: Context) -> Result<Response<Body>, HandlerError> {
        self.run_internal(req)
            .map_err(failure::Error::from)
            .map_err(failure::Error::into)
    }
}

impl RocketHandler {
    fn run_internal(&self, req: Request) -> Result<Response<Body>, RocketLambError> {
        let local_req = self.create_rocket_request(req)?;
        let local_res = local_req.dispatch();
        to_lambda_response(local_res)
    }

    fn create_rocket_request(&self, req: Request) -> Result<LocalRequest, RocketLambError> {
        let client = &self.0;
        let method = to_rocket_method(req.method())?;
        let uri = get_path_and_query(&req);
        let mut local_req = client.req(method, uri);
        for (name, value) in req.headers() {
            match value.to_str() {
                Ok(v) => local_req.add_header(Header::new(name.to_string(), v.to_string())),
                Err(_) => return Err(invalid_request!("invalid value for header '{}'", name)),
            }
        }
        local_req.set_body(req.into_body());
        Ok(local_req)
    }
}

fn to_lambda_response(mut local_res: LocalResponse) -> Result<Response<Body>, RocketLambError> {
    let mut builder = Response::builder();
    builder.status(local_res.status().code);
    for h in local_res.headers().iter() {
        builder.header(&h.name.to_string(), &h.value.to_string());
    }

    // TODO support binary response bodies
    let body = match local_res.body() {
        Some(b) => Body::Text(
            b.into_string()
                .ok_or_else(|| invalid_response!("could not read response body as UTF-8 text"))?,
        ),
        None => Body::Empty,
    };

    builder
        .body(body)
        .map_err(|e| invalid_response!("error creating Response: {}", e))
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

fn to_rocket_method(method: &http::Method) -> Result<Method, RocketLambError> {
    Ok(match *method {
        http::Method::GET => Method::Get,
        http::Method::PUT => Method::Put,
        http::Method::POST => Method::Post,
        http::Method::DELETE => Method::Delete,
        http::Method::OPTIONS => Method::Options,
        http::Method::HEAD => Method::Head,
        http::Method::TRACE => Method::Trace,
        http::Method::CONNECT => Method::Connect,
        http::Method::PATCH => Method::Patch,
        _ => return Err(invalid_request!("unknown method '{}'", method)),
    })
}
