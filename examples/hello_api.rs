#![feature(proc_macro_hygiene, decl_macro)]

use lambda_http::lambda;
use rocket::{get, routes};
use rocket_lamb::RocketHandler;

#[get("/")]
pub fn hello() -> &'static str {
    "Hello world!"
}

fn main() {
    let rocket = rocket::ignite().mount("/", routes![hello]);
    let handler = RocketHandler::new(rocket).unwrap();
    lambda!(handler);
}
