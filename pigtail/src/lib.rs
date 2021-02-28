#![allow(clippy::unnecessary_wraps)] // FIXME: Should get redone with better errors
use crate::types::QueueOperation;
use actix_web::{
    http::{header, Method},
    App, HttpResponse, Json, Path, State,
};
use anyhow::Result;
use hybrid_clocks::{Timestamp, WallT};
use lazy_static::lazy_static;
use log::{debug, info};
use potboiler_common::{clock, db, get_raw_timestamp, types::Log};
use serde_derive::Deserialize;
use serde_json::{self, Map, Value};
use std::{env, io::Cursor, ops::Deref};
use time::Duration;
use uuid::{self, Uuid};

mod types;

lazy_static! {
    static ref SERVER_URL: String = env::var("SERVER_URL").expect("Needed SERVER_URL");
    static ref HOST: String = env::var("HOST").unwrap_or_else(|_| "localhost".to_string());
    static ref PORT: u16 = u16::from_str_radix(&env::var("PORT").unwrap_or_else(|_| "8000".to_string()), 10).unwrap();
}

#[derive(Deserialize)]
struct NewLogResponse {
    id: Uuid,
}

fn add_queue_operation(op: &QueueOperation) -> actix_web::Result<NewLogResponse> {
    let client = reqwest::Client::new();
    let mut res = client.post(SERVER_URL.deref()).json(op).send().expect("sender ok");
    assert_eq!(res.status(), reqwest::StatusCode::CREATED);
    Ok(res.json().unwrap())
}

fn create_queue(op: Json<types::QueueCreate>) -> actix_web::Result<HttpResponse> {
    let name = op.name.clone();
    match add_queue_operation(&QueueOperation::Create(op.into_inner())) {
        Ok(_) => {
            let new_url = format!("/queue/{}", &name);
            Ok(HttpResponse::Created().header(header::LOCATION, new_url).finish())
        }
        Err(val) => Err(val),
    }
}

#[derive(Deserialize)]
struct NamedQueueRoute {
    queue_name: String,
}

fn delete_queue(path: Path<NamedQueueRoute>) -> actix_web::Result<HttpResponse> {
    add_queue_operation(&QueueOperation::Delete(path.queue_name.clone()))?;
    Ok(HttpResponse::Ok().finish())
}

fn row_to_state(row: &db::Row) -> Result<types::QueueState> {
    let raw_state: String = row.get("state");
    // FIXME: format! bit is a hacky workaround for https://github.com/serde-rs/serde/issues/251
    Ok(serde_json::from_str(&format!("\"{}\"", raw_state)).unwrap())
}

fn parse_progress<F>(state: &AppState, progress: &types::QueueProgress, should_update: F) -> Result<HttpResponse>
where
    F: Fn(&types::QueueState, &Timestamp<WallT>) -> Option<(Timestamp<WallT>, String)>,
{
    let conn = state.pool.get().unwrap();
    let results = conn.query(&format!(
        "SELECT task_name, state, hlc_tstamp from {} where id='{}'",
        &progress.queue_name, &progress.id
    ))?;
    if results.is_empty() {
        Ok(HttpResponse::NotFound().body(format!("No queue item {} in {}", &progress.id, &progress.queue_name)))
    } else {
        let row = results.get(0);
        let state = row_to_state(&row)?;
        let hlc_tstamp: Vec<u8> = row.get("hlc_tstamp");
        let when = Timestamp::read_bytes(Cursor::new(hlc_tstamp))?;
        if let Some((log_when, status)) = should_update(&state, &when) {
            let raw_timestamp = get_raw_timestamp(&log_when)?;
            conn.execute(&format!(
                "UPDATE {} set hlc_tstamp='{}', worker='{}', state='{}' where id='{}'",
                &progress.queue_name,
                &raw_timestamp.sql(),
                &progress.worker_id,
                &status,
                &progress.id
            ))?;
            Ok(HttpResponse::NoContent().finish())
        } else {
            Ok(HttpResponse::Conflict().body("Out of date change"))
        }
    }
}

fn new_event(log: Json<Log>, state: State<AppState>) -> actix_web::Result<HttpResponse> {
    info!("log: {:?}", log);
    let log_when = log.when;
    state.clock.observe_timestamp(log_when);
    let op = serde_json::from_value::<QueueOperation>(log.data.clone())?;
    info!("op: {:?}", op);
    let conn = state.pool.get().unwrap();
    match op {
        QueueOperation::Create(create) => {
            info!("create: {:?}", create);
            let qc = types::QueueConfig {
                timeout_ms: create.timeout_ms,
            };
            match conn.execute(&format!(
                "INSERT INTO queues (key, config) VALUES('{}', '{}')",
                &create.name,
                &serde_json::to_value(&qc)?
            )) {
                Ok(_) => {
                    conn.execute(&format!(
                        "CREATE TABLE IF NOT EXISTS {} (id UUID PRIMARY KEY, task_name \
                         VARCHAR(2083) NOT NULL, state VARCHAR(8) NOT NULL, info JSONB NOT \
                         NULL, hlc_tstamp BYTEA NOT NULL, worker UUID NULL)",
                        &create.name
                    ))
                    .unwrap();
                }
                Err(db::Error::UniqueViolation) => {}
                Err(err) => Err(err).unwrap(),
            };
        }
        QueueOperation::Add(add) => {
            info!("add: {:?}", add);
            let raw_timestamp = get_raw_timestamp(&log.when)?;
            conn.execute(&format!(
                "INSERT INTO {} (id, task_name, state, info, hlc_tstamp) VALUES('{}', '{}', \
                 '{}', '{}', {})",
                add.queue_name,
                &log.id,
                &add.task_name,
                "pending",
                &serde_json::to_value(&add.info)?,
                &raw_timestamp.sql()
            ))
            .unwrap();
        }
        QueueOperation::Progress(progress) => {
            info!("progress: {:?}", progress);
            return Ok(parse_progress(state.deref(), &progress, |state, when| {
                if state == &types::QueueState::Pending || (state == &types::QueueState::Working && log_when > *when) {
                    Some((log_when, String::from("working")))
                } else {
                    None
                }
            })
            .unwrap());
        }
        QueueOperation::Done(done) => {
            info!("done: {:?}", done);
            return Ok(parse_progress(state.deref(), &done, |state, when| {
                if state != &types::QueueState::Done || (state == &types::QueueState::Done && log_when > *when) {
                    Some((log_when, String::from("done")))
                } else {
                    None
                }
            })
            .unwrap());
        }
        QueueOperation::Delete(queue_name) => {
            //let trans = conn.transaction()?;
            let trans = conn;
            trans.execute(&format!("DROP TABLE IF EXISTS {}", queue_name)).unwrap();
            trans
                .execute(&format!("DELETE FROM queues where key={}", &queue_name))
                .unwrap();
            //trans.commit()?;
        }
    };
    Ok(HttpResponse::NoContent().finish())
}

fn get_queue_items(state: State<AppState>, path: Path<NamedQueueRoute>) -> actix_web::Result<HttpResponse> {
    let conn = state.pool.get().unwrap();
    let config_row = conn
        .query(&format!("select config from queues where key='{}'", &path.queue_name))
        .unwrap();
    if config_row.is_empty() {
        return Ok(HttpResponse::NotFound().body(format!("No queue {}", path.queue_name)));
    }
    let config: types::QueueConfig = serde_json::from_value(config_row.get(0).get("config"))?;
    let results = conn
        .query(&format!(
            "select id, task_name, state, hlc_tstamp from {}",
            &path.queue_name
        ))
        .unwrap();
    let mut queue = Map::new();
    let now = state.clock.get_timestamp().time.as_timespec();
    let max_diff = Duration::milliseconds(config.timeout_ms);
    for row in &results {
        let id: Uuid = row.get("id");
        let mut state = row_to_state(&row).unwrap();
        if state == types::QueueState::Done {
            continue;
        }
        if state == types::QueueState::Working {
            let hlc_tstamp: Vec<u8> = row.get("hlc_tstamp");
            let when = Timestamp::read_bytes(Cursor::new(hlc_tstamp))?;
            let diff = now - when.time.as_timespec();
            if diff > max_diff {
                debug!("{} is out of date, so marking as pending", id);
                state = types::QueueState::Pending;
            }
        }
        let item = types::QueueListItem {
            task_name: row.get("task_name"),
            state,
        };
        queue.insert(id.to_string(), serde_json::to_value(&item)?);
    }
    let value = Value::Object(queue);
    Ok(HttpResponse::Ok().json(value))
}

#[derive(Deserialize)]
struct QueueItemRoute {
    queue_name: String,
    id: String,
}

fn get_queue_item(path: Path<QueueItemRoute>, state: State<AppState>) -> actix_web::Result<HttpResponse> {
    let conn = state.pool.get().unwrap();
    let results = conn
        .query(&format!(
            "select task_name, state, info, worker from {} where id='{}'",
            &path.queue_name, &path.id
        ))
        .unwrap();
    if results.is_empty() {
        Ok(HttpResponse::NotFound().body(format!("No queue item {} in {}", path.id, path.queue_name)))
    } else {
        let row = results.get(0);
        let item = types::QueueItem {
            task_name: row.get("task_name"),
            state: row_to_state(&row).unwrap(),
            info: row.get("info"),
            worker: row.get("worker"),
        };
        Ok(HttpResponse::Ok().json(item))
    }
}

fn add_queue_item(json: Json<Value>, path: Path<NamedQueueRoute>) -> actix_web::Result<HttpResponse> {
    let mut json_mut = json.into_inner();
    let map = json_mut.as_object_mut().unwrap();
    map.insert("queue_name".to_string(), serde_json::to_value(&path.queue_name)?);
    let op = serde_json::from_value::<types::QueueAdd>(json_mut)?;
    match add_queue_operation(&QueueOperation::Add(op)) {
        Ok(val) => {
            let new_url = format!("http://{}:8000/queue/{}/{}", HOST.deref(), &path.queue_name, &val.id);
            Ok(HttpResponse::Created().header(header::LOCATION, new_url).finish())
        }
        Err(val) => Err(val),
    }
}

fn build_queue_progress(json: Json<Value>, path: &Path<QueueItemRoute>) -> Result<types::QueueProgress> {
    let mut json_mut = json.into_inner();
    let map = json_mut.as_object_mut().unwrap();
    map.insert("queue_name".to_string(), serde_json::to_value(&path.queue_name)?);
    map.insert("id".to_string(), serde_json::to_value(&path.id)?);
    Ok(serde_json::from_value::<types::QueueProgress>(json_mut)?)
}

fn progress_queue_item(
    state: State<AppState>,
    json: Json<Value>,
    path: Path<QueueItemRoute>,
) -> actix_web::Result<HttpResponse> {
    let op = build_queue_progress(json, &path).unwrap();
    match add_queue_operation(&QueueOperation::Progress(op)) {
        Ok(_) => get_queue_item(path, state),
        Err(val) => Err(val),
    }
}

fn finish_queue_item(json: Json<Value>, path: Path<QueueItemRoute>) -> actix_web::Result<HttpResponse> {
    let op = build_queue_progress(json, &path).unwrap();
    match add_queue_operation(&QueueOperation::Done(op)) {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(val) => Err(val),
    }
}

fn make_queue_table(conn: &db::Connection) {
    conn.execute("CREATE TABLE IF NOT EXISTS queues (key VARCHAR(1024) PRIMARY KEY, config JSONB NOT NULL)")
        .unwrap();
}

#[derive(Debug, Clone)]
pub struct AppState {
    clock: clock::SyncClock,
    pool: db::Pool,
}

impl AppState {
    pub fn new(pool: db::Pool) -> Result<AppState> {
        let clock = clock::SyncClock::new();
        let conn = pool.get().unwrap();
        make_queue_table(&conn);
        Ok(AppState { clock, pool })
    }
}

pub fn app_router(state: AppState) -> Result<App<AppState>> {
    Ok(App::with_state(state)
        .resource("/create", |r| r.method(Method::POST).with(create_queue))
        .resource("/event", |r| r.method(Method::POST).with(new_event))
        .resource("/queue/{queue_name}", |r| {
            r.method(Method::GET).with(get_queue_items);
            r.method(Method::POST).with(add_queue_item);
            r.method(Method::DELETE).with(delete_queue);
        })
        .resource("/queue/{queue_name}/{id}", |r| {
            r.method(Method::GET).with(get_queue_item);
            r.method(Method::PUT).with(progress_queue_item);
            r.method(Method::DELETE).with(finish_queue_item);
        }))
}

pub fn register() {
    let client = reqwest::Client::new();
    let mut map = serde_json::Map::new();
    map.insert(
        "url".to_string(),
        serde_json::Value::String(format!("http://{}:{}/event", HOST.deref(), PORT.deref())),
    );
    let res = client
        .post(&format!("{}/register", SERVER_URL.deref()))
        .json(&map)
        .send()
        .expect("Register ok");
    assert_eq!(res.status(), reqwest::StatusCode::CREATED);
}
