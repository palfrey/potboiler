use iron::{self, status::Status, Chain, Headers};
use iron_test::{self, request, response::extract_body_to_string};
use persistent::{self, Read as PRead};
use potboiler;
use potboiler_common::{self, server_id};
use pretty_assertions::assert_eq;
use serde_json::Value;
use serial_test_derive::serial;

fn test_setup() -> Chain {
    let pool = potboiler::db_setup().unwrap();
    pool.get().unwrap().execute("delete from log").unwrap();
    let mut router = potboiler::app_router(pool).unwrap();
    router.link_before(PRead::<server_id::ServerId>::one(server_id::test()));
    return router;
}

#[test]
#[serial]
fn test_empty_log() {
    let router = test_setup();
    let response = request::get("http://localhost:8000/log", Headers::new(), &router).unwrap();
    assert_eq!(response.status.unwrap(), Status::Ok);
    let result = extract_body_to_string(response);
    assert_eq!(result, "{}");
}

#[test]
#[serial]
fn test_create() {
    let router = test_setup();
    let mut response = request::post("http://localhost:8000/log", Headers::new(), "{}", &router).unwrap();
    assert_eq!(response.status.unwrap(), Status::Created);
    response = request::get("http://localhost:8000/log", Headers::new(), &router).unwrap();
    assert_eq!(response.status.unwrap(), Status::Ok);
    let result = extract_body_to_string(response);
    let v: Value = serde_json::from_str(&result).unwrap();
    assert!(v
        .as_object()
        .unwrap()
        .contains_key("feedface-dead-feed-face-deadfacedead"));
}
