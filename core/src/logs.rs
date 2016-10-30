use clock;
use hybrid_clocks;
use iron;
use iron::modifiers::Redirect;
use iron::prelude::{Response, IronResult, Request, IronError};
use iron::status;
use nodes;
use notifications;
use persistent;
use postgres::rows::{Row, RowIndex};
use postgres::types::FromSql;
use potboiler_common::{db, server_id};
use router::Router;
use serde_json::{self, Value, Map};
use serde_types::Log;
use std::io::{Cursor, Read};
use std::ops::Deref;
use std::sync::Arc;
use uuid::Uuid;

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

pub fn log_lasts(req: &mut Request) -> IronResult<Response> {
    log_status(req, "SELECT id, owner from log WHERE next is null")
}

pub fn log_firsts(req: &mut Request) -> IronResult<Response> {
    log_status(req, "SELECT id, owner from log WHERE prev is null")
}

fn json_from_body(mut req: &mut Request) -> Result<serde_json::Value, serde_json::Error> {
    let body_string = {
        let mut body = String::new();
        req.body.read_to_string(&mut body).expect("could read from body");
        body
    };
    return match serde_json::de::from_str(&body_string) {
        Ok(val) => Ok(val),
        Err(err) => Err(err),
    };
}

pub fn new_log(mut req: &mut Request) -> IronResult<Response> {
    let conn = get_pg_connection!(&req);
    let json: Value = match json_from_body(req) {
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
    let log = Log {
        id: id,
        owner: server_id.clone(),
        prev: previous,
        next: None,
        when: when,
        data: json.clone(),
    };
    nodes::insert_log(&conn, &log);
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

pub fn other_log(mut req: &mut Request) -> IronResult<Response> {
    let json = json_from_body(req).unwrap();
    let log: Log = serde_json::from_value(json).unwrap();
    let conn = get_pg_connection!(&req);
    let existing = conn.query("SELECT id from log WHERE id=$1 limit 1", &[&log.id])
        .expect("bad existing query");
    if existing.is_empty() {
        nodes::insert_log(&conn, &log);
        let log_arc = Arc::new(log);
        notifications::notify_everyone(req, log_arc.clone());
        nodes::notify_everyone(req, log_arc.clone());
    } else {
        info!("Told about new log item ({}) I already have", log.id);
    }
    Ok(Response::with((status::Ok, "Added")))
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

pub fn get_log(req: &mut Request) -> IronResult<Response> {
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
