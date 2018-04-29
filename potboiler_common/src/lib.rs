// #![deny(missing_debug_implementations, missing_copy_implementations,
//         warnings,
//         trivial_numeric_casts,
//         unstable_features,
//         unused, future_incompatible)]

extern crate iron;
extern crate r2d2;
extern crate uuid;
extern crate serde_json;
extern crate hybrid_clocks;
extern crate router;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;
extern crate diesel;
extern crate r2d2_diesel;
extern crate byteorder;

pub mod db;
pub mod server_id;
pub mod types;
pub mod clock;
pub mod pg;

use hybrid_clocks::{Timestamp, WallT};
use iron::prelude::{IronError, Request};
use iron::status;
use std::io::Read;

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
    Ok(Some(String::from(json.get("url").unwrap().as_str().unwrap())))
}

pub fn get_req_key<T: Into<String>>(req: &Request, key: T) -> Option<String> {
    req.extensions
        .get::<router::Router>()
        .unwrap()
        .find(&key.into())
        .map(|s| s.to_string())
}

pub fn get_raw_timestamp(timestamp: &Timestamp<WallT>) -> Vec<u8> {
    let mut raw_timestamp: Vec<u8> = Vec::new();
    timestamp.write_bytes(&mut raw_timestamp).unwrap();
    return raw_timestamp;
}

pub fn iron_str_error<T: std::error::Error + std::marker::Send + 'static>(se: T) -> iron::IronError {
    let desc = format!("{:?}", se);
    return IronError::new(se, (status::BadRequest, desc));
}
