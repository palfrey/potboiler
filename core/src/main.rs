#[macro_use]
extern crate schemamama;
extern crate schemamama_postgres;
extern crate postgres;
mod schema;

#[macro_use]
extern crate log;
extern crate log4rs;

extern crate iron;
extern crate router;
extern crate logger;

use iron::modifiers::Redirect;
use iron::prelude::*;
use iron::status;
use logger::Logger;
use router::Router;

use std::env;
use std::io::Read;
use std::ops::Deref;

extern crate hyper;
extern crate url;

extern crate uuid;
use uuid::Uuid;
extern crate serde;
extern crate serde_json;
use serde_json::{Map, Value};
extern crate hybrid_clocks;
use std::io::Cursor;

extern crate r2d2;
extern crate r2d2_postgres;
extern crate persistent;
use persistent::Read as PRead;
use persistent::State;
use postgres::rows::{Row, RowIndex};
use postgres::types::FromSql;

#[macro_use]
extern crate potboiler_common;
use potboiler_common::db;
use potboiler_common::server_id;

mod serde_types;
use serde_types::Log;
mod notifications;
mod clock;
mod nodes;
use std::sync::Arc;

fn log_status<T: Into<String>>(req: &mut Request, stmt: T) -> IronResult<Response> {
    let conn = get_pg_connection!(&req);
    let stmt = conn.prepare(&stmt.into()).expect("prepare failure");
    let mut logs = Map::new();
    for row in &stmt.query(&[]).expect("last select works") {
        let id: Uuid = row.get("id");
        let owner: Uuid = row.get("owner");
        logs.insert(owner.to_string(), serde_json::to_value(&id.to_string()));
    }
    let value = Value::Object(logs);
    Ok(Response::with((status::Ok, serde_json::ser::to_string(&value).unwrap())))
}

fn log_lasts(req: &mut Request) -> IronResult<Response> {
    log_status(req, "SELECT id, owner from log WHERE next is null")
}

fn log_firsts(req: &mut Request) -> IronResult<Response> {
    log_status(req, "SELECT id, owner from log WHERE prev is null")
}

fn new_log(mut req: &mut Request) -> IronResult<Response> {
    let conn = get_pg_connection!(&req);
    let body_string = {
        let mut body = String::new();
        req.body.read_to_string(&mut body).expect("could read from body");
        body
    };
    let json: Value = match serde_json::de::from_str(&body_string) {
        Ok(val) => val,
        Err(err) => return Err(IronError::new(err, (status::BadRequest, "Bad JSON"))),
    };
    let id = Uuid::new_v4();
    let hyphenated = id.hyphenated().to_string();
    let server_id = get_server_id!(&req).deref().clone();
    let stmt = conn.prepare("SELECT id from log WHERE next is null and owner = $1 LIMIT 1")
        .expect("prepare failure");
    let results = stmt.query(&[&server_id]).expect("last select works");
    let previous = if results.is_empty() {
        None
    } else {
        let row = results.get(0);
        let id: Uuid = row.get("id");
        Some(id)
    };
    let when = clock::get_timestamp(&mut req);
    conn.execute("UPDATE log set next = $1 where owner = $2 and next is null",
                 &[&id, &server_id])
        .expect("update worked");
    let mut raw_timestamp: Vec<u8> = Vec::new();
    when.write_bytes(&mut raw_timestamp).unwrap();
    conn.execute("INSERT INTO log (id, owner, data, prev, hlc_tstamp) VALUES ($1, $2, $3, $4, $5)",
                 &[&id, &server_id, &json, &previous, &raw_timestamp])
        .expect("insert worked");
    let log = Log {
        id: id,
        owner: server_id.clone(),
        prev: previous,
        next: None,
        when: when,
        data: json.clone(),
    };
    let log_arc = Arc::new(log);
    notifications::notify_everyone(req, log_arc.clone());
    nodes::notify_everyone(req, log_arc.clone());
    let new_url = {
        let req_url = req.url.clone();
        let base_url = req_url.into_generic_url();
        base_url.join(&format!("/log/{}", &hyphenated)).expect("join url works")
    };
    Ok(Response::with((status::Created,
                       Redirect(iron::Url::from_generic_url(new_url).expect("URL parsed ok")))))
}

fn get_with_null<I, T>(row: &Row, index: I) -> Option<T>
    where I: RowIndex,
          T: FromSql
{
    match row.get_opt(index) {
        Some(val) => {
            match val {
                Ok(val) => Some(val),
                Err(_) => None,
            }
        }
        None => None,
    }
}

fn get_log(req: &mut Request) -> IronResult<Response> {
    let query = req.extensions
        .get::<Router>()
        .unwrap()
        .find("entry_id")
        .unwrap_or("/")
        .to_string();
    let query_id = match Uuid::parse_str(&query) {
        Ok(val) => val,
        Err(_) => {
            return Ok(Response::with((status::NotFound, format!("No log {}", query))));
        }
    };
    let conn = get_pg_connection!(&req);
    let stmt = conn.prepare("SELECT owner, next, prev, data, hlc_tstamp from log where id=$1")
        .expect("prepare failure");
    let results = stmt.query(&[&query_id]).expect("bad query");
    if results.is_empty() {
        Ok(Response::with((status::NotFound, format!("No log {}", query))))
    } else {
        let row = results.get(0);
        let hlc_tstamp: Vec<u8> = row.get("hlc_tstamp");
        let when = hybrid_clocks::Timestamp::read_bytes(Cursor::new(hlc_tstamp)).unwrap();
        let log = Log {
            id: query_id,
            owner: row.get("owner"),
            prev: get_with_null(&row, "prev"),
            next: get_with_null(&row, "next"),
            data: row.get("data"),
            when: when,
        };
        Ok(Response::with((status::Ok, serde_json::to_string(&log).unwrap())))
    }
}

fn main() {
    log4rs::init_file("log.yaml", Default::default()).unwrap();
    let db_url: &str = &env::var("DATABASE_URL").expect("Needed DATABASE_URL");
    let pool = db::get_pool(db_url);
    let conn = pool.get().unwrap();
    schema::up(&conn).unwrap();
    let (logger_before, logger_after) = Logger::new(None);
    let mut router = Router::new();
    router.get("/log", log_lasts);
    router.post("/log", new_log);
    router.get("/log/first", log_firsts);
    router.get("/log/:entry_id", get_log);
    router.post("/log/register", notifications::log_register);
    router.post("/log/deregister", notifications::log_deregister);
    router.get("/nodes", nodes::node_list);
    router.post("/nodes", nodes::node_add);
    router.delete("/nodes", nodes::node_remove);
    let mut chain = Chain::new(router);
    chain.link_before(logger_before);
    chain.link_after(logger_after);
    let mut notifiers = Vec::new();
    let stmt = conn.prepare("select url from notifications").expect("prepare failure");
    for row in &stmt.query(&[]).expect("notifications select works") {
        let url: String = row.get("url");
        notifiers.push(url);
    }
    chain.link_before(State::<notifications::Notifications>::one(notifiers));
    chain.link_before(State::<nodes::Nodes>::one(nodes::initial_nodes(pool.clone())));
    chain.link_before(State::<clock::Clock>::one(hybrid_clocks::Clock::wall()));
    chain.link_before(PRead::<server_id::ServerId>::one(server_id::setup()));
    chain.link(PRead::<db::PostgresDB>::both(pool));
    info!("Potboiler booted");
    Iron::new(chain).http("0.0.0.0:8000").unwrap();
}
