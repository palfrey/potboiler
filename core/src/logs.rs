use crate::{nodes, AppState};
use actix_web::{HttpRequest, HttpResponse, Json, Path, State};
use hybrid_clocks;
use log::info;
use potboiler_common::{db, types::Log};
use serde_derive::Serialize;
use serde_json::{self, Map, Value};
use std::{io::Cursor, sync::Arc};
use uuid::Uuid;

fn log_status<S: Into<String>>(req: HttpRequest<AppState>, stmt: S) -> HttpResponse {
    let conn = req.state().pool.get().unwrap();
    let mut logs = Map::new();
    for row in conn.query(&stmt.into()).expect("last select works").iter() {
        let id: Uuid = row.get("id");
        let owner: Uuid = row.get("owner");
        logs.insert(owner.to_string(), serde_json::to_value(&id.to_string()).unwrap());
    }
    let value = Value::Object(logs);
    HttpResponse::Ok().json(value)
}

pub fn log_lasts(req: HttpRequest<AppState>) -> HttpResponse {
    log_status(req, "select id, owner from log where next is null")
}

pub fn log_firsts(req: HttpRequest<AppState>) -> HttpResponse {
    log_status(req, "select id, owner from log where prev is null")
}

#[derive(Serialize)]
struct NewLogResponse {
    id: Uuid,
}

pub fn new_log(state: State<AppState>, json: Json<serde_json::Value>) -> HttpResponse {
    let conn = state.pool.get().unwrap();
    let id = Uuid::new_v4();
    let hyphenated = id.hyphenated().to_string();
    let when = state.clock.get_timestamp();
    let server_id = state.server_id;
    let results = conn
        .query(&format!(
            "select id from log where next is null and owner = '{}' limit 1",
            &server_id
        ))
        .expect("last select works");
    let previous = if results.is_empty() {
        None
    } else {
        let row = results.get(0);
        let id: Uuid = row.get("id");
        Some(id)
    };
    let log = Log {
        id,
        owner: server_id,
        prev: previous,
        next: None,
        when,
        data: json.clone(),
    };
    nodes::insert_log(&conn, &log).unwrap();
    let log_arc = Arc::new(log);
    state.notifications.notify_everyone(&log_arc);
    nodes::notify_everyone(state, &log_arc);
    let new_url = format!("/log/{}", &hyphenated);
    HttpResponse::Created()
        .header(actix_web::http::header::LOCATION, new_url.to_string())
        .json(NewLogResponse { id })
}

pub fn other_log(log: Json<Log>, state: State<AppState>) -> HttpResponse {
    let conn = state.pool.get().unwrap();
    let existing = conn
        .query(&format!("select id from log where id = {} limit 1", &log.id))
        .expect("bad existing query");
    if existing.is_empty() {
        nodes::insert_log(&conn, &log).unwrap();
        let log_arc = Arc::new(log.into_inner());
        state.notifications.notify_everyone(&log_arc);
        nodes::notify_everyone(state, &log_arc);
    } else {
        info!("Told about new log item ({}) I already have", log.id);
    }
    HttpResponse::Ok().reason("Added").finish()
}

fn get_with_null<T>(row: &db::Row, index: &str) -> Option<T>
where
    T: db::FromSql,
{
    match row.get_opt(index) {
        Some(val) => match val {
            Ok(val) => Some(val),
            Err(_) => None,
        },
        None => None,
    }
}

pub fn get_log(query: Path<String>, state: State<AppState>) -> HttpResponse {
    let query_id = match Uuid::parse_str(&query) {
        Ok(val) => val,
        // &format!("No log {}", query)
        Err(_) => return HttpResponse::NotFound().reason("No log").finish(),
    };
    let conn = state.pool.get().unwrap();
    let results = conn
        .query(&format!(
            "select owner, next, prev, data from log where id = '{}'",
            query_id
        ))
        .unwrap();
    if results.is_empty() {
        // &format!("No log {}", query)
        HttpResponse::NotFound().reason("No log").finish()
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
            when,
        };
        HttpResponse::Ok().json(log)
    }
}
