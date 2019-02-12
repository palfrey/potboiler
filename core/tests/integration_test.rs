use iron::{self, status::Status, Chain, Headers};
use iron_test::{self, request, response::extract_body_to_string};
use persistent::{self, Read as PRead};
use potboiler;
use potboiler_common::{self, server_id};
use pretty_assertions::assert_eq;
use serde_json::{json, Value};
use serial_test_derive::serial;

fn test_setup() -> Chain {
    let pool = potboiler::db_setup().unwrap();
    let conn = pool.get().unwrap();
    conn.execute("delete from log").unwrap();
    conn.execute("delete from nodes").unwrap();
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

#[test]
#[serial]
fn test_register() {
    let router = test_setup();
    let args = json!({
        "url": "http://foo"
    });
    let mut response = request::post(
        "http://localhost:8000/log/register",
        Headers::new(),
        &args.to_string(),
        &router,
    )
    .unwrap();
    assert_eq!(response.status.unwrap(), Status::Created);
    response = request::post(
        "http://localhost:8000/log/register",
        Headers::new(),
        &args.to_string(),
        &router,
    )
    .unwrap();
    assert_eq!(response.status.unwrap(), Status::Created);
}

#[test]
#[serial]
fn test_deregister() {
    let router = test_setup();
    let args = json!({
        "url": "http://bar"
    });
    let mut response = request::post(
        "http://localhost:8000/log/deregister",
        Headers::new(),
        &args.to_string(),
        &router,
    )
    .unwrap();
    assert_eq!(response.status.unwrap(), Status::NotFound);
    response = request::post(
        "http://localhost:8000/log/register",
        Headers::new(),
        &args.to_string(),
        &router,
    )
    .unwrap();
    assert_eq!(response.status.unwrap(), Status::Created);
    response = request::post(
        "http://localhost:8000/log/deregister",
        Headers::new(),
        &args.to_string(),
        &router,
    )
    .unwrap();
    assert_eq!(response.status.unwrap(), Status::NoContent);
}

#[test]
#[serial]
fn test_list_nodes() {
    let router = test_setup();
    let response = request::get("http://localhost:8000/nodes", Headers::new(), &router).unwrap();
    assert_eq!(response.status.unwrap(), Status::Ok);
    let result = extract_body_to_string(response);
    assert_eq!(result, "[]");
}

#[test]
#[serial]
fn test_add_nodes() {
    let router = test_setup();
    let args = json!({
        "url": "http://bar"
    });
    let mut response = request::post(
        "http://localhost:8000/nodes",
        Headers::new(),
        &args.to_string(),
        &router,
    )
    .unwrap();
    assert_eq!(response.status.unwrap(), Status::Created);
    response = request::get("http://localhost:8000/nodes", Headers::new(), &router).unwrap();
    assert_eq!(response.status.unwrap(), Status::Ok);
    let result = extract_body_to_string(response);
    assert_eq!(result, "[\"http://bar\"]");
}
