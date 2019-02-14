use iron::{self, status::Status, Chain, Headers};
use iron_test::{self, request, response::extract_body_to_string};
use kv;
use mockito;
use persistent::{self, Read as PRead};
use potboiler_common::{self, http_client, server_id};
use pretty_assertions::assert_eq;
use serde_json::json;
use serial_test_derive::serial;
use std::env;
use hybrid_clocks::{Clock as HClock};

fn test_setup() -> Chain {
    let _ = env_logger::try_init();
    let register_mock = mockito::mock("POST", "/register").with_status(201).create();
    env::set_var("SERVER_URL", mockito::SERVER_URL);
    let pool = kv::db_setup().unwrap();
    let conn = pool.get().unwrap();
    conn.execute("delete from log").unwrap();
    conn.execute("delete from nodes").unwrap();
    conn.execute("delete from _config").unwrap();
    let mut router = kv::app_router(pool).unwrap();
    router.link_before(PRead::<server_id::ServerId>::one(server_id::test()));
    let client = reqwest::Client::new();
    kv::register(&client).unwrap();
    register_mock.assert();
    http_client::set_client(&mut router, client);
    return router;
}

#[test]
#[serial]
fn test_empty_table() {
    let router = test_setup();
    let response = request::get("http://localhost:8000/kv/_config/test", Headers::new(), &router).unwrap();
    assert_eq!(response.status.unwrap(), Status::NotFound);
    let result = extract_body_to_string(response);
    assert_eq!(result, "No such key 'test'");
}

#[test]
#[serial]
fn test_no_such_table() {
    let router = test_setup();
    let response = request::get("http://localhost:8000/kv/test/foo", Headers::new(), &router).unwrap();
    assert_eq!(response.status.unwrap(), Status::NotFound);
    let result = extract_body_to_string(response);
    assert_eq!(result, "No such table 'test'");
}

#[test]
#[serial]
fn test_create_table() {
    let create_req = json!({
        "table": "_config",
        "key": "test",
        "op": "set",
        "change":{"crdt":"LWW"}
    });
    let create_mock = mockito::mock("POST", "/")
        .with_status(201)
        .match_body(mockito::Matcher::Json(create_req.clone()))
        .create();
    let router = test_setup();
    let mut response = request::get("http://localhost:8000/kv/test/foo", Headers::new(), &router).unwrap();
    let result = extract_body_to_string(response);
    assert_eq!(result, "No such table 'test'");

    let args = json!({
        "op": "set",
        "change": {"crdt": "LWW"}
    });
    response = request::post(
        "http://localhost:8000/kv/_config/test",
        Headers::new(),
        &args.to_string(),
        &router,
    )
    .unwrap();
    assert_eq!(response.status.unwrap(), Status::Ok);
    create_mock.assert();

    let log_req = json!({
        "id": uuid::Uuid::new_v4(),
        "owner": "feedface-dead-feed-face-deadfacedead",
        "when": HClock::wall().now(),
        "data": create_req
    });
    response = request::post(
        "http://localhost:8000/kv/event",
        Headers::new(),
        &log_req.to_string(),
        &router,
    )
    .unwrap();
    assert_eq!(response.status.unwrap(), Status::NoContent);

    response = request::get("http://localhost:8000/kv/test/foo", Headers::new(), &router).unwrap();
    let result = extract_body_to_string(response);
    assert_eq!(result, "No such key 'foo'");
}
