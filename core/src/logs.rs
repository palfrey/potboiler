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

fn notify_everyone(state: &State<AppState>, log_arc: &Arc<Log>) {
    // always notify all nodes
    nodes::notify_everyone(state, log_arc);

    // notifiers however require dependencies
    for dep in &log_arc.dependencies {
        if read_log(&state, dep).is_none() {
            info!("Missing dependency {} for {}", dep, log_arc.id);
            return;
        }
    }
    state.notifications.notify_everyone(log_arc);

    // Now get the logs that depend on this log entry
    let conn = state.pool.get().unwrap();
    let deps: Vec<Uuid> = conn
        .query(&format!(
            "select id from dependency where depends_on = '{}'",
            &log_arc.id
        ))
        .unwrap()
        .iter()
        .map(|r| r.get("id"))
        .collect();
    for dep in &deps {
        if let Some(other_log) = read_log(&state, dep) {
            // i.e. `log` is a dep of `other_log`, so start telling others about other_log
            let other_log_arc = Arc::new(other_log);
            // note we only need to tell notifiers, as nodes already know
            state.notifications.notify_everyone(&other_log_arc);
        }
    }
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
    notify_everyone(&state, &log_arc);
    let new_url = format!("/log/{}", &hyphenated);
    Ok(HttpResponse::Created()
        .header(actix_web::http::header::LOCATION, new_url)
        .json(NewLogResponse { id }))
}

pub fn other_log(log: Json<Log>, state: State<AppState>) -> HttpResponse {
    if log.owner == state.server_id {
        return HttpResponse::BadRequest().body("Attempted new log entry with local owner id");
    }
    let conn = state.pool.get().unwrap();
    let existing = conn
        .query(&format!("select id from log where id = '{}' limit 1", &log.id))
        .expect("bad existing query");
    if existing.is_empty() {
        nodes::insert_log(&conn, &log).unwrap();
        let log_arc = Arc::new(log.into_inner());
        notify_everyone(&state, &log_arc);
    } else {
        info!("Told about new log item ({}) I already have", log.id);
    }
    HttpResponse::Ok().body("Added")
}

fn read_log(state: &State<AppState>, id: &Uuid) -> Option<Log> {
    let conn = state.pool.get().unwrap();
    let results = conn
        .query(&format!(
            "select owner, next, prev, data, hlc_tstamp from log where id = '{}'",
            id
        ))
        .unwrap();
    if results.is_empty() {
        None
    } else {
        let row = results.get(0);
        let hlc_tstamp: Vec<u8> = row.get("hlc_tstamp");
        let when = hybrid_clocks::Timestamp::read_bytes(Cursor::new(hlc_tstamp)).unwrap();
        let deps = conn
            .query(&format!("select depends_on from dependency where id = '{}'", id))
            .unwrap()
            .iter()
            .map(|r| r.get("depends_on"))
            .collect();
        Some(Log {
            id: *id,
            owner: row.get("owner"),
            prev: row.get_with_null("prev"),
            next: row.get_with_null("next"),
            data: row.get("data"),
            when,
            dependencies: deps,
        })
    }
}

pub fn get_log(query: Path<String>, state: State<AppState>) -> HttpResponse {
    let query_id = match Uuid::parse_str(&query) {
        Ok(val) => val,
        Err(_) => return HttpResponse::NotFound().body(&format!("No log {}", query)),
    };
    match read_log(&state, &query_id) {
        None => HttpResponse::NotFound().body(&format!("No log {}", query)),
        Some(log) => HttpResponse::Ok().json(log),
    }
}
