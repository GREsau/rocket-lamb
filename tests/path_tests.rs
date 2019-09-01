#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use lambda_http::{Body, Handler, Request};
use lambda_runtime::Context;
use rocket::http::uri::Origin;
use rocket_lamb::{BasePathBehaviour, RocketExt};
use std::error::Error;
use std::fs::File;

#[catch(404)]
fn not_found(req: &rocket::Request) -> String {
    req.uri().to_string()
}

#[get("/path")]
fn get_path<'r>(origin: &'r Origin<'r>) -> &'r str {
    origin.path()
}

fn make_rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![get_path])
        .register(catchers![not_found])
}

fn get_request(json_file: &str) -> Result<Request, Box<dyn Error>> {
    let file = File::open(format!("tests/requests/{}.json", json_file))?;
    Ok(lambda_http::request::from_reader(file)?)
}

macro_rules! test_case {
    ($name:ident, $file:expr, $status:expr, $path:expr) => {
        test_case!($name, RemountAndInclude, $file, $status, $path);
    };
    ($name:ident, $path_behaviour:ident, $file:expr, $status:expr, $path:expr) => {
        #[test]
        fn $name() -> Result<(), Box<dyn Error>> {
            let mut handler = make_rocket()
                .lambda()
                .base_path_behaviour(BasePathBehaviour::$path_behaviour)
                .into_handler();

            let req = get_request($file)?;
            let res = handler.run(req, Context::default())?;

            assert_eq!(res.status(), $status);
            assert_eq!(*res.body(), Body::Text($path.to_string()));
            Ok(())
        }
    };
}

test_case!(api_gateway, "path_api_gateway", 200, "/Prod/path/");
test_case!(
    api_gateway_include_base,
    Include,
    "path_api_gateway",
    404,
    "/Prod/path/"
);
test_case!(
    api_gateway_exclude_base,
    Exclude,
    "path_api_gateway",
    200,
    "/path/"
);

test_case!(custom_domain, "path_custom_domain", 200, "/path/");
test_case!(
    custom_domain_include_empty_base,
    Include,
    "path_custom_domain",
    200,
    "/path/"
);
test_case!(
    custom_domain_exclude_empty_base,
    Exclude,
    "path_custom_domain",
    200,
    "/path/"
);

test_case!(
    custom_domain_with_base_path,
    "path_custom_domain_with_base",
    200,
    "/base-path/path/"
);
test_case!(
    custom_domain_with_base_path_include,
    Include,
    "path_custom_domain_with_base",
    404,
    "/base-path/path/"
);
test_case!(
    custom_domain_with_base_path_exclude,
    Exclude,
    "path_custom_domain_with_base",
    200,
    "/path/"
);

test_case!(application_load_balancer, "path_alb", 200, "/path/");
test_case!(
    application_load_balancer_include_empty_base,
    Include,
    "path_alb",
    200,
    "/path/"
);
test_case!(
    application_load_balancer_exclude_empty_base,
    Exclude,
    "path_alb",
    200,
    "/path/"
);
