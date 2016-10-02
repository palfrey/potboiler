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
use url::Url;

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
use postgres::error::SqlState;
use postgres::rows::{Row, RowIndex};
use postgres::types::FromSql;

#[macro_use]
extern crate potboiler_common;
use potboiler_common::db;
use potboiler_common::server_id;

include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));

mod notifications;
mod clock;

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
    let notifications = notifications::get_notifications_list(req);
    let client = hyper::client::Client::new();
    for notifier in notifications {
        let log = Log {
            id: id,
            owner: server_id.clone(),
            prev: previous,
            next: None,
            when: when,
            data: json.clone(),
        };
        debug!("Notifying {:?}", notifier);
        let res = client.post(&notifier)
            .body(&serde_json::ser::to_string(&log).unwrap())
            .send()
            .unwrap();
        if res.status != hyper::status::StatusCode::NoContent {
            warn!("Failed to notify {:?}: {:?}", &notifier, res.status);
        }
    }
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
        let when = Timestamp::read_bytes(Cursor::new(hlc_tstamp)).unwrap();
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

fn url_from_body(req: &mut Request) -> Result<Option<String>, IronError> {
    let body_string = {
        let mut body = String::new();
        req.body.read_to_string(&mut body).expect("could read from body");
        body
    };
    let json: Value = match serde_json::de::from_str(&body_string) {
        Ok(val) => val,
        Err(err) => return Err(IronError::new(err, (status::BadRequest, "Bad JSON"))),
    };
    Ok(Some(json.find("url").unwrap().as_string().unwrap().to_string()))
}

fn log_register(req: &mut Request) -> IronResult<Response> {
    let conn = get_pg_connection!(&req);
    let url = url_from_body(req).unwrap().unwrap();
    debug!("Registering {:?}", url);
    match Url::parse(&url) {
        Err(err) => Err(IronError::new(err, (status::BadRequest, "Bad URL"))),
        Ok(_) => {
            match conn.execute("INSERT INTO notifications (url) VALUES ($1)", &[&url]) {
                Ok(_) => {
                    notifications::insert_notifier(req, &url);
                    Ok(Response::with((status::NoContent)))
                }
                Err(err) => {
                    if let postgres::error::Error::Db(dberr) = err {
                        match dberr.code {
                            SqlState::UniqueViolation => Ok(Response::with((status::NoContent))),
                            _ => Err(IronError::new(dberr, (status::BadRequest, "Some other error"))),
                        }
                    } else {
                        Err(IronError::new(err, (status::BadRequest, "Some other error")))
                    }
                }
            }
        }
    }
}

fn log_deregister(req: &mut Request) -> IronResult<Response> {
    let conn = get_pg_connection!(&req);
    conn.execute("DELETE from notifications where url = $1",
                 &[&url_from_body(req).unwrap().unwrap()])
        .expect("delete worked");
    Ok(Response::with((status::NoContent)))
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
    router.post("/log/register", log_register);
    router.post("/log/deregister", log_deregister);
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
    chain.link_before(State::<clock::Clock>::one(hybrid_clocks::Clock::wall()));
    chain.link_before(PRead::<server_id::ServerId>::one(server_id::setup()));
    chain.link(PRead::<db::PostgresDB>::both(pool));
    info!("Potboiler booted");
    Iron::new(chain).http("localhost:8000").unwrap();
}
