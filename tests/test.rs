#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use lambda_http::{Body, Handler, Request, Response};
use lambda_runtime::Context;
use rocket_lamb::{ResponseType, RocketHandler};
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

fn make_rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![upper])
        .register(catchers![not_found])
}

fn get_request(json_file: &'static str) -> Result<Request, Box<dyn Error>> {
    let file = File::open(json_file)?;
    Ok(lambda_http::request::from_reader(file)?)
}

mod test {
    use super::*;

    #[test]
    fn ok() -> Result<(), Box<dyn Error>> {
        let mut handler = RocketHandler::new(make_rocket())?;

        let req = get_request("tests/request_upper.json")?;
        let res = handler.run(req, Context::default())?;

        assert_eq!(res.status(), 200);
        assert_header(&res, "content-type", "text/plain; charset=utf-8");
        assert_eq!(*res.body(), Body::Text("ONE, TWO, THREE".to_string()));
        Ok(())
    }

    #[test]
    fn ok_binary_default() -> Result<(), Box<dyn Error>> {
        let mut handler =
            RocketHandler::new(make_rocket())?.default_response_type(ResponseType::Binary);

        let req = get_request("tests/request_upper.json")?;
        let res = handler.run(req, Context::default())?;

        assert_eq!(res.status(), 200);
        assert_header(&res, "content-type", "text/plain; charset=utf-8");
        assert_eq!(
            *res.body(),
            Body::Binary("ONE, TWO, THREE".to_owned().into_bytes())
        );
        Ok(())
    }

    #[test]
    fn ok_binary() -> Result<(), Box<dyn Error>> {
        let mut handler =
            RocketHandler::new(make_rocket())?.response_type("TEXT/PLAIN", ResponseType::Binary);

        let req = get_request("tests/request_upper.json")?;
        let res = handler.run(req, Context::default())?;

        assert_eq!(res.status(), 200);
        assert_header(&res, "content-type", "text/plain; charset=utf-8");
        assert_eq!(
            *res.body(),
            Body::Binary("ONE, TWO, THREE".to_owned().into_bytes())
        );
        Ok(())
    }

    #[test]
    fn not_found() -> Result<(), Box<dyn Error>> {
        let mut handler = RocketHandler::new(make_rocket())?;

        let req = get_request("tests/request_not_found.json")?;
        let res = handler.run(req, Context::default())?;

        assert_eq!(res.status(), 404);
        assert_eq!(res.headers().contains_key("content-type"), false);
        assert!(res.body().is_empty(), "Response body should be empty");
        Ok(())
    }

    fn assert_header(res: &Response<Body>, name: &str, value: &str) {
        let values = res.headers().get_all(name).iter().collect::<Vec<_>>();
        assert_eq!(values.len(), 1, "Header {} should have 1 value", name);
        assert_eq!(values[0], value);
    }
}
