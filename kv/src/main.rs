#[macro_use]
extern crate log;
extern crate log4rs;

extern crate iron;
extern crate logger;
extern crate router;
use iron::prelude::*;
use iron::status;
use logger::Logger;
use router::Router;

extern crate persistent;
use persistent::Read as PRead;

#[macro_use]
extern crate potboiler_common;
use potboiler_common::db;
use potboiler_common::server_id;

extern crate serde_json;
extern crate hyper;

use std::env;
use std::io::Read;

include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));

static SERVER_URL: &'static str = "http://localhost:8000/log";

fn get_req_key<T: Into<String>>(req: &Request, key: T) -> Option<String> {
    req.extensions
        .get::<Router>()
        .unwrap()
        .find(&key.into())
        .map(|s| s.to_string())
}

fn get_key(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "get_key")))
}

fn update_key(req: &mut Request) -> IronResult<Response> {
    let body_string = {
        let mut body = String::new();
        req.body.read_to_string(&mut body).expect("could read from body");
        body
    };
    let mut json: serde_json::Value = match serde_json::de::from_str(&body_string) {
        Ok(val) => val,
        Err(err) => return Err(IronError::new(err, (status::BadRequest, "Bad JSON"))),
    };
    let map = json.as_object_mut().unwrap();
    map.insert("table".to_string(),
               serde_json::to_value(&get_req_key(req, "table").unwrap()));
    map.insert("key".to_string(),
               serde_json::to_value(&get_req_key(req, "key").unwrap()));

    let client = hyper::client::Client::new();

    let res = client.post(SERVER_URL)
        .body(&serde_json::ser::to_string(&map).unwrap())
        .send()
        .unwrap();
    assert_eq!(res.status, hyper::status::StatusCode::Created);
    Ok(Response::with((status::Ok, "update_key")))
}

fn new_event(req: &mut Request) -> IronResult<Response> {
    let body_string = {
        let mut body = String::new();
        req.body.read_to_string(&mut body).expect("could read from body");
        body
    };
    let json: serde_json::Value = match serde_json::de::from_str(&body_string) {
        Ok(val) => val,
        Err(err) => return Err(IronError::new(err, (status::BadRequest, "Bad JSON"))),
    };
    info!("body: {:?}", json);
    let data = json.find("data").unwrap();
    let change: Change = serde_json::from_value(data.clone()).unwrap();
    info!("change: {:?}", change);
    Ok(Response::with(status::NoContent))
}

fn main() {
    log4rs::init_file("log.yaml", Default::default()).unwrap();
    let client = hyper::client::Client::new();

    let mut map: serde_json::Map<String, String> = serde_json::Map::new();
    map.insert("url".to_string(),
               "http://localhost:8001/kv/event".to_string());
    let res = client.post(&format!("{}/register", SERVER_URL))
        .body(&serde_json::ser::to_string(&map).unwrap())
        .send()
        .unwrap();
    assert_eq!(res.status, hyper::status::StatusCode::NoContent);

    let db_url: &str = &env::var("DATABASE_URL").expect("Needed DATABASE_URL");
    let pool = db::get_pool(db_url);
    // let conn = pool.get().unwrap();
    //    schema::up(&conn).unwrap();
    let (logger_before, logger_after) = Logger::new(None);
    let mut router = Router::new();
    router.get("/kv/:table/:key", get_key);
    router.post("/kv/:table/:key", update_key);
    router.post("/kv/event", new_event);
    let mut chain = Chain::new(router);
    chain.link_before(logger_before);
    chain.link_after(logger_after);
    chain.link_before(PRead::<server_id::ServerId>::one(server_id::setup()));
    chain.link(PRead::<db::PostgresDB>::both(pool));
    info!("Potboiler-kv booted");
    Iron::new(chain).http("localhost:8001").unwrap();
}
