#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use lambda_http::{Body, Handler, Request};
use lambda_runtime::Context;
use rocket::http::uri::Origin;
use rocket_lamb::RocketExt;
use std::error::Error;
use std::fs::File;

#[get("/path")]
fn get_path<'r>(origin: &'r Origin<'r>) -> &'r str {
    origin.path()
}

fn make_rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![get_path])
}

fn get_request(json_file: &'static str) -> Result<Request, Box<dyn Error>> {
    let file = File::open(format!("tests/requests/{}.json", json_file))?;
    Ok(lambda_http::request::from_reader(file)?)
}

#[test]
fn api_gateway_url_with_stage() -> Result<(), Box<dyn Error>> {
    let mut handler = make_rocket().lambda().into_handler();

    let req = get_request("path_api_gateway")?;
    let res = handler.run(req, Context::default())?;

    assert_eq!(res.status(), 200);
    assert_eq!(*res.body(), Body::Text("/Prod/path/".to_string()));
    Ok(())
}

#[test]
fn api_gateway_url_without_stage() -> Result<(), Box<dyn Error>> {
    let mut handler = make_rocket()
        .lambda()
        .include_api_gateway_base_path(false)
        .into_handler();

    let req = get_request("path_api_gateway")?;
    let res = handler.run(req, Context::default())?;

    assert_eq!(res.status(), 200);
    assert_eq!(*res.body(), Body::Text("/path/".to_string()));
    Ok(())
}

#[test]
fn custom_domain() -> Result<(), Box<dyn Error>> {
    let mut handler = make_rocket().lambda().into_handler();

    let req = get_request("path_custom_domain")?;
    let res = handler.run(req, Context::default())?;

    assert_eq!(res.status(), 200);
    assert_eq!(*res.body(), Body::Text("/path/".to_string()));
    Ok(())
}

#[test]
#[ignore]
fn custom_domain_with_base_path() -> Result<(), Box<dyn Error>> {
    let mut handler = make_rocket().lambda().into_handler();

    let req = get_request("path_custom_domain_with_base")?;
    let res = handler.run(req, Context::default())?;

    assert_eq!(res.status(), 200);
    assert_eq!(*res.body(), Body::Text("/base-path/path/".to_string()));
    Ok(())
}

#[test]
fn application_load_balancer() -> Result<(), Box<dyn Error>> {
    let mut handler = make_rocket().lambda().into_handler();

    let req = get_request("path_alb")?;
    let res = handler.run(req, Context::default())?;

    assert_eq!(res.status(), 200);
    assert_eq!(*res.body(), Body::Text("/path/".to_string()));
    Ok(())
}
