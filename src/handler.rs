use crate::error::RocketLambError;
use crate::ResponseType;
use lambda_http::{Body, Handler, Request, RequestExt, Response};
use lambda_runtime::{error::HandlerError, Context};
use rocket::http::{uri::Uri, Header};
use rocket::local::{Client, LocalRequest, LocalResponse};
use std::collections::HashMap;

/// A Lambda handler for API Gateway events that processes requests using `Rocket`.
pub struct RocketHandler {
    pub(super) client: Client,
    pub(super) default_response_type: ResponseType,
    pub(super) response_types: HashMap<String, ResponseType>,
}

impl Handler<Response<Body>> for RocketHandler {
    fn run(&mut self, req: Request, _ctx: Context) -> Result<Response<Body>, HandlerError> {
        self.process_request(req)
            .map_err(failure::Error::from)
            .map_err(failure::Error::into)
    }
}

impl RocketHandler {
    fn process_request(&self, req: Request) -> Result<Response<Body>, RocketLambError> {
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
            .and_then(|ct| self.response_types.get(&ct.to_lowercase()))
            .map(|rt| *rt)
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
