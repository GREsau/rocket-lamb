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

pub struct RocketHandler(Client);

impl RocketHandler {
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

    fn run_internal(&self, req: Request) -> Result<Response<Body>, RocketLambError> {
        let local_req = self.create_rocket_request(req)?;
        let local_res = local_req.dispatch();
        to_lambda_response(local_res)
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
