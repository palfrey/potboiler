extern crate iron;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate uuid;
#[macro_use]
extern crate log;

pub mod db;
pub mod server_id;
pub mod string_error;

use iron::prelude::{IronError, Request};
use iron::status;
use std::io::Read;
extern crate serde_json;

pub fn url_from_body(req: &mut Request) -> Result<Option<String>, IronError> {
    let body_string = {
        let mut body = String::new();
        req.body.read_to_string(&mut body).expect("could read from body");
        body
    };
    let json: serde_json::Value = match serde_json::de::from_str(&body_string) {
        Ok(val) => val,
        Err(err) => return Err(IronError::new(err, (status::BadRequest, "Bad JSON"))),
    };
    Ok(Some(String::from(json.find("url").unwrap().as_str().unwrap())))
}
