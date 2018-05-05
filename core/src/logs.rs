use hybrid_clocks;
use iron;
use iron::modifiers::Redirect;
use iron::prelude::{IronError, IronResult, Request, Response};
use iron::status;
use nodes;
use notifications;
use persistent;
use potboiler_common::{clock, db, server_id};
use potboiler_common::types::Log;
use router::Router;
use serde_json::{self, Map, Value};
use std::io::{Cursor, Read};
use std::ops::Deref;
use std::sync::Arc;
use url::Url;
use uuid::Uuid;
use deuterium::{NamedField, ToIsPredicate, TableDef, Selectable, ToIsNullPredicate, Queryable};

error_chain! {
    foreign_links {
        SerdeError(serde_json::Error);
    }
}

iron_error_from!();

pub struct LogTable;

impl LogTable {
    pub fn table() -> TableDef {
        TableDef::new("log")
    }

    pub fn id() -> NamedField<Uuid> {
        return NamedField::<Uuid>::field_of("id", &LogTable::table());
    }

    pub fn owner() -> NamedField<Uuid> {
        return NamedField::<Uuid>::field_of("owner", &LogTable::table());
    }

    pub fn next() -> NamedField<Option<Uuid>> {
        return NamedField::<Option<Uuid>>::field_of("next", &LogTable::table());
    }

    pub fn prev() -> NamedField<Option<Uuid>> {
        return NamedField::<Option<Uuid>>::field_of("prev", &LogTable::table());
    }

    pub fn data() -> NamedField<Uuid> {
        return NamedField::<Uuid>::field_of("data", &LogTable::table());
    }

    pub fn hlc_tstamp() -> NamedField<Uuid> {
        return NamedField::<Uuid>::field_of("hlc_tstamp", &LogTable::table());
    }
}

fn log_status<T: Into<String>>(req: &mut Request, stmt: T) -> IronResult<Response> {
    let conn = get_db_connection!(&req);
    let mut logs = Map::new();
    for row in conn.query(&stmt.into()).expect("last select works").iter() {
        let id: Uuid = row.get("id");
        let owner: Uuid = row.get("owner");
        logs.insert(owner.to_string(),
                    serde_json::to_value(&id.to_string()).unwrap());
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

fn json_from_body(req: &mut Request) -> Result<serde_json::Value> {
    let body_string = {
        let mut body = String::new();
        req.body.read_to_string(&mut body).expect("could read from body");
        body
    };
    return serde_json::de::from_str(&body_string).map_err(|e| e.into());
}

#[derive(Serialize)]
struct NewLogResponse {
    id: Uuid
}

pub fn new_log(mut req: &mut Request) -> IronResult<Response> {
    let conn = get_db_connection!(&req);
    let json: Value = match json_from_body(req) {
        Ok(val) => val,
        Err(err) => return Err(IronError::new(err, (status::BadRequest, "Bad JSON"))),
    };
    let id = Uuid::new_v4();
    let hyphenated = id.hyphenated().to_string();
    let server_id = get_server_id!(&req).deref().clone();
    let results = conn.dquery(&LogTable::table()
        .select(&[&LogTable::id()])
        .where_(LogTable::next().is_null()).and(LogTable::owner().is(server_id))
        .limit(1))
        .expect("last select works");
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
    nodes::insert_log(&conn, &log)?;
    let log_arc = Arc::new(log);
    notifications::notify_everyone(req, log_arc.clone());
    nodes::notify_everyone(req, log_arc.clone());
    let new_url = {
        let req_url = req.url.clone();
        let base_url: Url = req_url.into();
        base_url.join(&format!("/log/{}", &hyphenated)).expect("join url works")
    };
    Ok(Response::with((status::Created,
                       serde_json::to_string(&NewLogResponse{id:id}).map_err(|e| Error::from_kind(ErrorKind::SerdeError(e)))?,
                       Redirect(iron::Url::from_generic_url(new_url).expect("URL parsed ok")))))
}

pub fn other_log(req: &mut Request) -> IronResult<Response> {
    let json = json_from_body(req).unwrap();
    let log: Log = serde_json::from_value(json).unwrap();
    let conn = get_db_connection!(&req);
    let existing = conn.dquery(&LogTable::table()
        .select(&[&LogTable::id()])
        .where_(LogTable::id().is(log.id))
        .limit(1))
        .expect("bad existing query");
    if existing.is_empty() {
        nodes::insert_log(&conn, &log)?;
        let log_arc = Arc::new(log);
        notifications::notify_everyone(req, log_arc.clone());
        nodes::notify_everyone(req, log_arc.clone());
    } else {
        info!("Told about new log item ({}) I already have", log.id);
    }
    Ok(Response::with((status::Ok, "Added")))
}

fn get_with_null<T>(row: &db::Row, index: &str) -> Option<T>
    where T: db::FromSql
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
    let conn = get_db_connection!(&req);

    let results = conn.dquery(&LogTable::table()
        .select(&[&LogTable::owner(), &LogTable::next(), &LogTable::prev(), &LogTable::data(), &LogTable::hlc_tstamp()])
        .where_(LogTable::id().is(query_id))).expect("log query issue");

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
