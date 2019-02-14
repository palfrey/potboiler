use iron::{self, status::Status, Chain, Headers, Iron, Listening};
use iron_test::{self, request, response::extract_body_to_string};
use kv;
use mockito;
use potboiler;
use persistent::{self, Read as PRead};
use potboiler_common::{self, http_client, server_id};
use pretty_assertions::assert_eq;
use serde_json::json;
use serial_test_derive::serial;
use std::env;
use hybrid_clocks::{Clock as HClock};
use std::net::TcpListener;
use log::info;

type IronServer = Result<Listening, potboiler::Error>;

fn get_available_port() -> Option<u16> {
    (8000..9000)
        .find(|port| port_is_available(*port))
}

fn port_is_available(port: u16) -> bool {
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn boot_potboiler() -> potboiler::Result<(IronServer,String)> {
    let pool = potboiler::db_setup()?;
    let mut chain = potboiler::app_router(pool)?;
    chain.link_before(PRead::<server_id::ServerId>::one(server_id::setup()));
    info!("Potboiler booted");
    let port = get_available_port().unwrap();
    let iron = Iron::new(chain)
        .http(("0.0.0.0", port))
        .map_err(|e| potboiler::Error::with_chain(e, potboiler::ErrorKind::IronError));
    return Ok((iron, format!("127.0.0.1:{}", port)));
}

fn test_setup() -> (IronServer, Chain) {
    let _ = env_logger::try_init();
    let (iron, potboiler_url) = boot_potboiler().unwrap();
    env::set_var("SERVER_URL", potboiler_url);
    let pool = kv::db_setup().unwrap();
    let conn = pool.get().unwrap();
    conn.execute("delete from log").unwrap();
    conn.execute("delete from nodes").unwrap();
    conn.execute("delete from _config").unwrap();
    let mut router = kv::app_router(pool).unwrap();
    router.link_before(PRead::<server_id::ServerId>::one(server_id::test()));
    let client = reqwest::Client::new();
    kv::register(&client).unwrap();
    http_client::set_client(&mut router, client);
    info!("Potboiler-kv booted");
    return (iron, router);
}

#[test]
#[serial]
fn test_empty_table() {
    let (_iron, router) = test_setup();
    let response = request::get("http://localhost:8000/kv/_config/test", Headers::new(), &router).unwrap();
    assert_eq!(response.status.unwrap(), Status::NotFound);
    let result = extract_body_to_string(response);
    assert_eq!(result, "No such key 'test'");
}

#[test]
#[serial]
fn test_no_such_table() {
    let (_iron, router) = test_setup();
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
    let (_iron, router) = test_setup();
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
