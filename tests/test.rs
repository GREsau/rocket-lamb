#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use lambda_http::{Body, Handler, Response};
use lambda_runtime::Context;
use rocket_lamb::RocketHandler;
use std::error::Error;
use std::fs::File;

#[catch(404)]
fn not_found() {}

#[post("/upper/<path>?<query>", data = "<body>")]
fn upper(path: String, query: String, body: String) -> String {
    format!(
        "{}, {}, {}",
        path.to_uppercase(),
        query.to_uppercase(),
        body.to_uppercase()
    )
}

mod test {
    use super::*;

    #[test]
    fn ok() -> Result<(), Box<dyn Error>> {
        let rocket = rocket::ignite()
            .mount("/", routes![upper])
            .register(catchers![not_found]);
        let mut handler = RocketHandler::new(rocket)?;

        let file = File::open("tests/request_upper.json")?;
        let req = lambda_http::request::from_reader(file)?;
        let response = handler.run(req, Context::default())?;

        assert_eq!(response.status(), 200);
        assert_header(&response, "content-type", "text/plain; charset=utf-8");
        assert_eq!(*response.body(), Body::Text("ONE, TWO, THREE".to_string()));
        Ok(())
    }

    #[test]
    fn not_found() -> Result<(), Box<dyn Error>> {
        let rocket = rocket::ignite()
            .mount("/", routes![upper])
            .register(catchers![not_found]);
        let mut handler = RocketHandler::new(rocket)?;

        let file = File::open("tests/request_not_found.json")?;
        let req = lambda_http::request::from_reader(file)?;
        let response = handler.run(req, Context::default())?;

        assert_eq!(response.status(), 404);
        assert_eq!(response.headers().contains_key("content-type"), false);
        assert!(response.body().is_empty(), "Response body should be empty");
        Ok(())
    }

    fn assert_header(res: &Response<Body>, name: &str, value: &str) {
        let values = res.headers().get_all(name).iter().collect::<Vec<_>>();
        assert_eq!(values.len(), 1, "Header {} should have 1 value", name);
        assert_eq!(values[0], value);
    }
}
