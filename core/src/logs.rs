use crate::{nodes, AppState};
use actix_web::{HttpRequest, HttpResponse, Json, Path, State};
use failure::{bail, Error, Fail};
use hybrid_clocks;
use log::info;
use potboiler_common::types::Log;
use serde_derive::Serialize;
use serde_json::{self, Map, Value};
use std::{io::Cursor, sync::Arc};
use url::form_urlencoded;
use uuid::{self, Uuid};

#[derive(Debug, Fail)]
enum LogsError {
    #[fail(display = "Bad query key: {}", name)]
    BadQueryKey { name: String },
    #[fail(display = "Bad dependency uuid")]
    BadDepUuid {
        #[cause]
        cause: uuid::ParseError,
    },
}

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

pub fn new_log(
    state: State<AppState>,
    json: Json<serde_json::Value>,
    req: HttpRequest<AppState>,
) -> Result<HttpResponse, Error> {
    let conn = state.pool.get().unwrap();
    let id = Uuid::new_v4();
    let hyphenated = id.hyphenated().to_string();
    let when = state.clock.get_timestamp();
    let server_id = state.server_id;
    let mut dependencies: Vec<Uuid> = Vec::new();
    for (name, value) in form_urlencoded::parse(req.query_string().as_bytes()) {
        if name != "dependency" {
            bail!(LogsError::BadQueryKey { name: name.to_string() });
        }
        dependencies.push(Uuid::parse_str(&value).map_err(|e| LogsError::BadDepUuid { cause: e })?);
    }
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
        dependencies,
    };
    nodes::insert_log(&conn, &log).unwrap();
    let log_arc = Arc::new(log);
    state.notifications.notify_everyone(&log_arc);
    nodes::notify_everyone(state, &log_arc);
    let new_url = format!("/log/{}", &hyphenated);
    Ok(HttpResponse::Created()
        .header(actix_web::http::header::LOCATION, new_url.to_string())
        .json(NewLogResponse { id }))
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
    HttpResponse::Ok().body("Added")
}

pub fn get_log(query: Path<String>, state: State<AppState>) -> HttpResponse {
    let query_id = match Uuid::parse_str(&query) {
        Ok(val) => val,
        Err(_) => return HttpResponse::NotFound().body(&format!("No log {}", query)),
    };
    let conn = state.pool.get().unwrap();
    let results = conn
        .query(&format!(
            "select owner, next, prev, data, hlc_tstamp from log where id = '{}'",
            query_id
        ))
        .unwrap();
    if results.is_empty() {
        HttpResponse::NotFound().body(&format!("No log {}", query))
    } else {
        let row = results.get(0);
        let hlc_tstamp: Vec<u8> = row.get("hlc_tstamp");
        let when = hybrid_clocks::Timestamp::read_bytes(Cursor::new(hlc_tstamp)).unwrap();
        let deps = conn
            .query(&format!("select depends_on from dependency where id = '{}'", query_id))
            .unwrap()
            .iter()
            .map(|r| r.get("depends_on"))
            .collect();
        let log = Log {
            id: query_id,
            owner: row.get("owner"),
            prev: row.get_with_null("prev"),
            next: row.get_with_null("next"),
            data: row.get("data"),
            when,
            dependencies: deps,
        };
        HttpResponse::Ok().json(log)
    }
}
