use actix_web::{server, test::TestServer};
use anyhow::{ensure, Result};

use potboiler_common::{server_id, test::wait_for_action, test::ServerThread};
use pretty_assertions::assert_eq;
use reqwest::{blocking::Client, StatusCode};
use serde_json::json;
use serial_test_derive::serial;
use std::env;
use thiserror::Error;

#[derive(Debug, Error)]
enum IntegrationError {
    #[error("IoError")]
    IoError {
        #[source]
        cause: std::io::Error,
    },
}

fn boot_potboiler() -> Result<ServerThread> {
    let _ = env_logger::try_init();
    let pool = potboiler::db_setup()?;
    let app_state = potboiler::AppState::new(pool, server_id::test()).unwrap();
    ServerThread::new({
        move || server::new(move || potboiler::app_router(app_state.clone()).unwrap()).bind("0.0.0.0:8000")
    })
    .map_err(|e| IntegrationError::IoError { cause: e }.into())
}

fn test_setup() -> Result<(ServerThread, TestServer)> {
    let _ = env_logger::try_init();
    let pool = kv::db_setup().unwrap();
    pool.wipe_db()?;
    let pb_server = boot_potboiler()?;
    env::set_var("SERVER_URL", "http://localhost:8000/log");
    let client = reqwest::blocking::Client::new();
    let app_state = kv::AppState::new(pool, client.clone()).unwrap();
    let kv_server = TestServer::with_factory(move || kv::app_router(app_state.clone()).unwrap());
    env::set_var("KV_ROOT", kv_server.url("/"));
    kv::register(&client).unwrap();
    Ok((pb_server, kv_server))
}

#[test]
#[serial]
fn test_empty_table() {
    let (_pb_server, kv_server) = test_setup().unwrap();
    let client = Client::new();
    let response = client.get(&kv_server.url("/kv/_config/test")).send().unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.text().unwrap(), "No such key 'test'");
}

#[test]
#[serial]
fn test_no_such_table() {
    let (_pb_server, kv_server) = test_setup().unwrap();
    let client = Client::new();
    let response = client.get(&kv_server.url("/kv/test/foo")).send().unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.text().unwrap(), "No such table 'test'");
}

#[test]
#[serial]
fn test_create_table() {
    let (_pb_server, kv_server) = test_setup().unwrap();
    let client = Client::new();
    let mut response = client.get(&kv_server.url("/kv/test/foo")).send().unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.text().unwrap(), "No such table 'test'");

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

    wait_for_action(|| {
        let r = client.get(&kv_server.url("/kv/test/foo")).send()?;
        ensure!(r.status() == StatusCode::NOT_FOUND, "Not found");
        ensure!(r.text().unwrap() == "No such key 'foo'", "No foo key");
        Ok(())
    })
    .unwrap();
}

#[test]
#[serial]
fn test_create_orset_table() {
    let (_pb_server, kv_server) = test_setup().unwrap();
    let client = Client::new();
    let args = json!({
        "op": "set",
        "change": {"crdt": "ORSET"}
    });
    let response = client
        .post(&kv_server.url("/kv/_config/test"))
        .json(&args)
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let new_key = json!({
        "op": "add",
        "change": {
            "key": "[key]",
            "item":"[item]",
            "metadata": "[metadata]"
        }
    });

    wait_for_action(|| {
        let mut response = client.post(&kv_server.url("/kv/test/foo")).json(&new_key).send()?;
        ensure!(response.status() == StatusCode::OK, "Not ok");
        response = client.get(&kv_server.url("/kv/test/foo")).send()?;
        ensure!(response.status() == StatusCode::OK, "Not ok");
        let text = response.text().unwrap();
        ensure!(
            text == "[{\"item\":\"[item]\",\"key\":\"[key]\",\"metadata\":\"[metadata]\"}]",
            text
        );
        Ok(())
    })
    .unwrap();
}
