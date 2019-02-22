use actix_web::{http::Method, test::TestServer, App, HttpRequest, HttpResponse};
use env_logger;
use hybrid_clocks::Clock;
use potboiler;
use potboiler_common::server_id;
use pretty_assertions::assert_eq;
use reqwest::{self, Client, StatusCode};
use serde_derive::Deserialize;
use serde_json::{json, Value};
use serial_test_derive::serial;
use std::ops::DerefMut;
use std::sync::{Arc, RwLock};
use std::{thread, time};
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

#[derive(Debug)]
struct RecordRequest {
    pub path: String,
    pub body: String,
    pub method: Method,
}

struct RecordServer {
    requests: Arc<RwLock<Vec<RecordRequest>>>,
    server: TestServer,
}

impl RecordServer {
    fn recording_server(requests: Arc<RwLock<Vec<RecordRequest>>>) -> App {
        App::new().default_resource(move |r| {
            r.route().with(move |(req, body): (HttpRequest, String)| {
                requests.clone().write().unwrap().deref_mut().push(RecordRequest {
                    path: req.path().to_string(),
                    body: body,
                    method: req.method().clone(),
                });
                HttpResponse::MethodNotAllowed()
            })
        })
    }

    pub fn new() -> RecordServer {
        let requests = Arc::new(RwLock::new(Vec::new()));
        RecordServer {
            requests: requests.clone(),
            server: TestServer::with_factory(move || RecordServer::recording_server(requests.clone())),
        }
    }
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

#[derive(Deserialize, Debug)]
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

fn create_log(dependencies: Option<Vec<Uuid>>) -> Value {
    let timestamp = Clock::wall().now();
    let id = uuid::Uuid::new_v4();
    let owner = uuid::Uuid::new_v4();
    let mut log = json!({
        "id": &id,
        "owner": &owner,
        "prev": null,
        "next": null,
        "when": timestamp,
        "data": {}
    });
    if let Some(deps) = dependencies {
        log.as_object_mut()
            .unwrap()
            .insert("dependencies".to_string(), json!(deps));
    }
    log
}

fn make_log(test_server: &TestServer, log: &Value) {
    let client = Client::new();
    let response = client.post(&test_server.url("/log/other")).json(log).send().unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

fn get_json_key(json: &Value, key: &str) -> String {
    json.get(key).unwrap().as_str().unwrap().to_string()
}

#[test]
#[serial]
fn test_other_log() {
    let test_server = test_setup();
    let log = create_log(None);
    make_log(&test_server, &log);
    let mut response = reqwest::get(&test_server.url("/log")).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let v: Value = dbg!(serde_json::from_str(&response.text().unwrap()).unwrap());
    let objv = v.as_object().unwrap();
    let owner = get_json_key(&log, "owner");
    assert!(objv.contains_key(&owner));
    assert_eq!(objv[&owner], get_json_key(&log, "id"));
}

fn register(test_server: &TestServer, url: &str) {
    let args = json!({ "url": url });
    let client = Client::new();
    let response = client
        .post(&test_server.url("/log/register"))
        .json(&args)
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[test]
#[serial]
fn test_register() {
    let test_server = test_setup();
    register(&test_server, "http://foo");
    // Duplicate should just succeed
    register(&test_server, "http://foo");
}

#[test]
#[serial]
fn test_create_blocked_dependency() {
    let test_server = test_setup();
    let log = create_log(None);
    let block_id = get_json_key(&log, "id");
    let client = Client::new();
    let rs = RecordServer::new();
    register(&test_server, &rs.server.url("/"));
    let mut response = client
        .post(&test_server.url(&format!("/log?dependency={}", &block_id)))
        .json(&{})
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let new_log: NewLogResponse = response.json().unwrap();

    // Pause to do things
    thread::sleep(time::Duration::from_millis(100));

    assert_eq!(rs.requests.read().unwrap().len(), 0, "{:?} {:?}", new_log, rs.requests);
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
