use actix_web::server;
use anyhow::Result;

use potboiler_common::{pg, server_id, test::ServerThread};
use regex::Regex;
use reqwest::{blocking::Client, header, StatusCode};
use serde_json::json;
use serial_test_derive::serial;
use std::{env, str};
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

fn test_setup() -> Result<(ServerThread, ServerThread)> {
    let _ = env_logger::try_init();
    let db_url: &str = &env::var("DATABASE_URL").expect("Needed DATABASE_URL");
    let pool = pg::get_pool(db_url).unwrap();
    pool.wipe_db()?;
    let pb_server = boot_potboiler()?;
    env::set_var("SERVER_URL", "http://localhost:8000/log");
    env::set_var("PORT", "8001");
    let app_state = pigtail::AppState::new(pool)?;
    let pigtail_server = ServerThread::new({
        move || server::new(move || pigtail::app_router(app_state.clone()).unwrap()).bind("0.0.0.0:8001")
    })?;
    pigtail::register();
    Ok((pb_server, pigtail_server))
}

#[test]
#[serial]
fn test_create_queue() {
    let (_pb_server, _pigtail_server) = test_setup().unwrap();
    let client = Client::new();
    let response = client
        .post("http://localhost:8001/create")
        .json(&json!({"name":"foo", "timeout_ms": 1000}))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[test]
#[serial]
fn test_add_entry() {
    let (_pb_server, _pigtail_server) = test_setup().unwrap();
    let client = Client::new();
    let mut response = client
        .post("http://localhost:8001/create")
        .json(&json!({"name":"foo", "timeout_ms": 1000}))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    response = client
        .post("http://localhost:8001/queue/foo")
        .json(&json!({"task_name":"hello_world", "info":{"foo":"bar"}}))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let re = Regex::new(r"/queue/foo/([a-z0-9-]+)").unwrap();
    let location = format!("{:?}", response.headers()[header::LOCATION]);
    assert!(re.is_match(&location), "{}", location);
}
