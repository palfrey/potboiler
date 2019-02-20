use actix_web::test::TestServer;
use env_logger;
use hybrid_clocks::Clock;
use potboiler;
use potboiler_common::server_id;
use pretty_assertions::assert_eq;
use reqwest::{Client, StatusCode};
use serde_derive::Deserialize;
use serde_json::{json, Value};
use serial_test_derive::serial;
use uuid::Uuid;

fn test_setup() -> TestServer {
    let _ = env_logger::try_init();
    let pool = potboiler::db_setup().unwrap();
    let conn = pool.get().unwrap();
    conn.execute("delete from dependency").unwrap();
    conn.execute("delete from log").unwrap();
    conn.execute("delete from nodes").unwrap();
    let app_state = potboiler::AppState::new(pool, server_id::test()).unwrap();
    return TestServer::with_factory(move || potboiler::app_router(app_state.clone()).unwrap());
}

#[test]
#[serial]
fn test_empty_log() {
    let test_server = test_setup();
    let client = Client::new();
    let mut response = client.get(&test_server.url("/log")).send().unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().unwrap(), "{}");
}

#[derive(Deserialize)]
struct NewLogResponse {
    id: Uuid,
}

#[test]
#[serial]
fn test_create() {
    let test_server = test_setup();
    let client = Client::new();
    let mut response = client.post(&test_server.url("/log")).json(&{}).send().unwrap();
    println!("{:?}", &response);
    assert_eq!(response.status(), StatusCode::CREATED);
    response = client.get(&test_server.url("/log")).send().unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let v: Value = dbg!(serde_json::from_str(&response.text().unwrap()).unwrap());
    assert!(v
        .as_object()
        .unwrap()
        .contains_key("feedface-dead-feed-face-deadfacedead"));
}

#[test]
#[serial]
fn test_create_dependency() {
    let test_server = test_setup();
    let client = Client::new();
    let mut response = client
        .post(&test_server.url("/log?dependency=feedface-dead-feed-face-deadfacedead"))
        .json(&{})
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let new_log: NewLogResponse = response.json().unwrap();
    response = client
        .get(&test_server.url(&format!("/log/{}", new_log.id)))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let v: Value = dbg!(serde_json::from_str(&response.text().unwrap()).unwrap());
    assert!(
        dbg!(v.as_object().unwrap().get("dependencies").unwrap().as_array().unwrap())
            .contains(&json!("feedface-dead-feed-face-deadfacedead"))
    );
}

#[test]
#[serial]
fn test_other_log() {
    let test_server = test_setup();
    let client = Client::new();
    let timestamp = Clock::wall().now();
    let id = uuid::Uuid::new_v4();
    let owner = uuid::Uuid::new_v4();
    let log = json!({
        "id": &id,
        "owner": &owner,
        "prev": null,
        "next": null,
        "when": timestamp,
        "data": {}
    });
    let mut response = client.post(&test_server.url("/log/other")).json(&log).send().unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    response = client.get(&test_server.url("/log")).send().unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let v: Value = dbg!(serde_json::from_str(&response.text().unwrap()).unwrap());
    let objv = v.as_object().unwrap();
    assert!(objv.contains_key(&owner.to_string()));
    assert_eq!(objv[&owner.to_string()], id.to_string());
}

#[test]
#[serial]
fn test_register() {
    let test_server = test_setup();
    let args = json!({
        "url": "http://foo"
    });
    let client = Client::new();
    let mut response = client
        .post(&test_server.url("/log/register"))
        .json(&args)
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    // Duplicate should just succeed
    response = client
        .post(&test_server.url("/log/register"))
        .json(&args)
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[test]
#[serial]
fn test_deregister() {
    let test_server = test_setup();
    let args = json!({
        "url": "http://bar"
    });
    let client = Client::new();
    let mut response = client
        .post(&test_server.url("/log/deregister"))
        .json(&args)
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    response = client
        .post(&test_server.url("/log/register"))
        .json(&args)
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    response = client
        .post(&test_server.url("/log/deregister"))
        .json(&args)
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[test]
#[serial]
fn test_list_nodes() {
    let test_server = test_setup();
    let mut response = reqwest::get(&test_server.url("/nodes")).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().unwrap(), "[]");
}

#[test]
#[serial]
fn test_add_nodes() {
    let test_server = test_setup();
    let args = json!({
        "url": "http://bar"
    });
    let client = Client::new();
    let mut response = client.post(&test_server.url("/nodes")).json(&args).send().unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    response = client.get(&test_server.url("/nodes")).send().unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().unwrap(), "[\"http://bar\"]");
}
