use actix_web::{server, test::TestServer};
use failure::Error;
use kv;
use potboiler;
use potboiler_common::{server_id, ServerThread};
use pretty_assertions::assert_eq;
use reqwest::{Client, StatusCode};
use serde_json::json;
use serial_test_derive::serial;
use std::env;

fn boot_potboiler() -> Result<ServerThread, std::io::Error> {
    let _ = env_logger::try_init();
    let pool = potboiler::db_setup().unwrap();
    let conn = pool.get().unwrap();
    conn.execute("delete from log").unwrap();
    conn.execute("delete from nodes").unwrap();
    conn.execute("delete from notifications").unwrap();
    let app_state = potboiler::AppState::new(pool, server_id::test()).unwrap();
    return ServerThread::new({
        move || server::new(move || potboiler::app_router(app_state.clone()).unwrap()).bind("0.0.0.0:8000")
    });
}

fn test_setup() -> Result<(ServerThread, TestServer), Error> {
    let _ = env_logger::try_init();
    let pool = kv::db_setup().unwrap();
    let conn = pool.get()?;
    conn.execute("delete from _config").unwrap();
    let pb_server = boot_potboiler()?;
    env::set_var("SERVER_URL", dbg!("http://localhost:8000/log"));
    let client = reqwest::Client::new();
    let app_state = kv::AppState::new(pool, client.clone()).unwrap();
    let kv_server = TestServer::with_factory(move || kv::app_router(app_state.clone()).unwrap());
    env::set_var("KV_ROOT", dbg!(kv_server.url("/")));
    kv::register(&client).unwrap();
    return Ok((pb_server, kv_server));
}

#[test]
#[serial]
fn test_empty_table() {
    let (_pb_server, kv_server) = test_setup().unwrap();
    let client = Client::new();
    let mut response = client.get(&kv_server.url("/kv/_config/test")).send().unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.text().unwrap(), "");
    //assert_eq!(result, "No such key 'test'");
}

#[test]
#[serial]
fn test_no_such_table() {
    let (_pb_server, kv_server) = test_setup().unwrap();
    let client = Client::new();
    let mut response = client.get(&kv_server.url("/kv/test/foo")).send().unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.text().unwrap(), "");
    //assert_eq!(result, "No such table 'test'");
}

#[test]
#[serial]
fn test_create_table() {
    let (_pb_server, kv_server) = test_setup().unwrap();
    let client = Client::new();
    let mut response = client.get(&kv_server.url("/kv/test/foo")).send().unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let args = json!({
        "op": "set",
        "change": {"crdt": "LWW"}
    });
    response = client
        .post(&kv_server.url("/kv/_config/test"))
        .json(&args)
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    response = client.get(&kv_server.url("/kv/test/foo")).send().unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    //assert_eq!(result, "No such key 'foo'");
}
