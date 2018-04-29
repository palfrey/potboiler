#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate iron;
extern crate persistent;
#[macro_use]
extern crate potboiler_common;
extern crate logger;
extern crate serde_json;
extern crate serde;
extern crate hyper;
extern crate router;
extern crate uuid;
extern crate hybrid_clocks;
extern crate time;
#[macro_use]
extern crate serde_derive;

use hybrid_clocks::{Timestamp, WallT};
use iron::modifiers::Redirect;
use iron::prelude::{Chain, Iron, IronError, IronResult, Request, Response};
use iron::status;
use logger::Logger;
use persistent::Read as PRead;
use postgres::error::SqlState;
use potboiler_common::{clock, db, get_raw_timestamp, iron_str_error};
use potboiler_common::string_error::StringError;
use potboiler_common::types::Log;
use serde_json::{Map, Value};
use std::env;
use std::io::Cursor;
use std::ops::Deref;
use time::Duration;
use types::QueueOperation;
use uuid::Uuid;

mod types;

pub type PostgresConnection = r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>;

lazy_static! {
    static ref SERVER_URL: String = env::var("SERVER_URL").expect("Needed SERVER_URL");
    static ref HOST: String = env::var("HOST").expect("Needed HOST");
}

fn string_from_body<T: std::io::Read>(body: &mut T) -> IronResult<String> {
    let mut result = String::new();
    try!(body.read_to_string(&mut result).map_err(iron_str_error));
    Ok(result)
}

fn json_from_body(req: &mut Request) -> IronResult<Value> {
    let body_string = try!(string_from_body(&mut req.body));
    let json: Value = match serde_json::de::from_str(&body_string) {
        Ok(val) => val,
        Err(err) => return Err(IronError::new(err, (status::BadRequest, "Bad JSON"))),
    };
    return Ok(json);
}

fn add_queue_operation(op: QueueOperation) -> IronResult<String> {
    let client = hyper::client::Client::new();
    let mut res = client.post(SERVER_URL.deref())
        .body(&serde_json::ser::to_string(&op).unwrap())
        .send()
        .expect("sender ok");
    assert_eq!(res.status, hyper::status::StatusCode::Created);
    let data = try!(string_from_body(&mut res));
    Ok(data)
}

fn create_queue(req: &mut Request) -> IronResult<Response> {
    let json = try!(json_from_body(req));
    let op = try!(serde_json::from_value::<types::QueueCreate>(json).map_err(iron_str_error));
    let name = op.name.clone();
    match add_queue_operation(QueueOperation::Create(op)) {
        Ok(_) => {
            let new_url = format!("http://{}:8000/queue/{}", HOST.deref(), &name);
            Ok(Response::with((status::Created,
                               Redirect(iron::Url::parse(&new_url).expect("URL parsed ok")))))
        }
        Err(val) => Err(val),
    }
}

fn delete_queue(req: &mut Request) -> IronResult<Response> {
    let queue_name = try!(get_queue_name(req));
    try!(add_queue_operation(QueueOperation::Delete(queue_name)));
    Ok(Response::with(status::Ok))
}

fn row_to_state(row: &postgres::rows::Row) -> IronResult<types::QueueState> {
    let raw_state: String = row.get("state");
    // FIXME: format! bit is a hacky workaround for https://github.com/serde-rs/serde/issues/251
    return serde_json::from_str(&format!("\"{}\"", raw_state)).map_err(iron_str_error);
}

fn parse_progress<F>(req: &mut Request,
                     progress: types::QueueProgress,
                     should_update: F)
                     -> IronResult<Response>
    where F: Fn(&types::QueueState, &Timestamp<WallT>) -> Option<(Timestamp<WallT>, String)>
{
    let conn = get_db_connection!(&req);
    let results = try!(conn.query(&format!("SELECT task_name, state, hlc_tstamp from {} where id=$1",
                        &progress.queue_name),
               &[&progress.id])
        .map_err(iron_str_error));
    if results.is_empty() {
        return Ok(Response::with((status::NotFound,
                                  format!("No queue item {} in {}", &progress.id, &progress.queue_name))));
    } else {
        let row = results.get(0);
        let state = try!(row_to_state(&row));
        let hlc_tstamp: Vec<u8> = row.get("hlc_tstamp");
        let when = try!(hybrid_clocks::Timestamp::read_bytes(Cursor::new(hlc_tstamp))
            .map_err(iron_str_error));
        if let Some((log_when, status)) = should_update(&state, &when) {
            let raw_timestamp = get_raw_timestamp(&log_when);
            try!(conn.execute(&format!("UPDATE {} set hlc_tstamp=$1, worker=$2, state=$3 where id=$4",
                                  &progress.queue_name),
                         &[&raw_timestamp, &progress.worker_id, &status, &progress.id])
                .map_err(iron_str_error));
            return Ok(Response::with(status::NoContent));
        } else {
            return Ok(Response::with((status::Conflict, "Out of date change")));
        }
    }
}

fn new_event(req: &mut Request) -> IronResult<Response> {
    let json = try!(json_from_body(req));
    let log = try!(serde_json::from_value::<Log>(json).map_err(iron_str_error));
    info!("log: {:?}", log);
    let log_when = log.when.clone();
    clock::observe_timestamp(&clock::get_clock(req), log_when);
    let op = try!(serde_json::from_value::<QueueOperation>(log.data).map_err(iron_str_error));
    info!("op: {:?}", op);
    let conn = get_db_connection!(&req);
    match op {
        QueueOperation::Create(create) => {
            info!("create: {:?}", create);
            let qc = types::QueueConfig { timeout_ms: create.timeout_ms };
            match conn.execute("INSERT INTO queues (key, config) VALUES($1, $2)",
                               &[&create.name, &serde_json::to_value(&qc).map_err(iron_str_error)?]) {
                Ok(_) => {
                    try!(conn.execute(
                        &format!("CREATE TABLE IF NOT EXISTS {} (id UUID PRIMARY KEY, task_name \
                                   VARCHAR(2083) NOT NULL, state VARCHAR(8) NOT NULL, info JSONB NOT \
                                   NULL, hlc_tstamp BYTEA NOT NULL, worker UUID NULL)",
                                  &create.name),
                         &[]).map_err(iron_str_error));
                }
                Err(err) => {
                    if let postgres::error::Error::Db(dberr) = err {
                        match dberr.code {
                            SqlState::UniqueViolation => {}
                            _ => return Err(iron_str_error(dberr)),
                        };
                    } else {
                        return Err(iron_str_error(err));
                    }
                }
            };

        }
        QueueOperation::Add(add) => {
            info!("add: {:?}", add);
            let raw_timestamp = get_raw_timestamp(&log.when);
            conn.execute(&format!("INSERT INTO {} (id, task_name, state, info, hlc_tstamp) VALUES($1, $2, \
                                   $3, $4, $5)",
                                  add.queue_name),
                         &[&log.id,
                           &add.task_name,
                           &String::from("pending"),
                           &serde_json::to_value(&add.info).map_err(iron_str_error)?,
                           &raw_timestamp])
                .unwrap();
        }
        QueueOperation::Progress(progress) => {
            info!("progress: {:?}", progress);
            return parse_progress(req, progress, |state, when| {
                if state == &types::QueueState::Pending ||
                   (state == &types::QueueState::Working && &log_when > when) {
                    Some((log_when, String::from("working")))
                } else {
                    None
                }
            });
        }
        QueueOperation::Done(done) => {
            info!("done: {:?}", done);
            return parse_progress(req, done, |state, when| {
                if state != &types::QueueState::Done ||
                   (state == &types::QueueState::Done && &log_when > when) {
                    Some((log_when, String::from("done")))
                } else {
                    None
                }
            });
        }
        QueueOperation::Delete(queue_name) => {
            let trans = try!(conn.transaction().map_err(iron_str_error));
            try!(trans.execute(&format!("DROP TABLE IF EXISTS {}", queue_name), &[]).map_err(iron_str_error));
            try!(trans.execute("DELETE FROM queues where key=$1", &[&queue_name]).map_err(iron_str_error));
            try!(trans.commit().map_err(iron_str_error));
        }
    };
    Ok(Response::with(status::NoContent))
}

fn get_req_key_with_iron_err(req: &mut Request, key: &str) -> IronResult<String> {
    Ok(try!(potboiler_common::get_req_key(req, key)
        .ok_or(iron_str_error(StringError::from(format!("No {}", key))))))
}

fn get_queue_name(req: &mut Request) -> IronResult<String> {
    get_req_key_with_iron_err(req, "queue_name")
}

fn get_queue_items(req: &mut Request) -> IronResult<Response> {
    let conn = get_db_connection!(&req);
    let queue_name = try!(get_queue_name(req));
    let config_row = try!(conn.query("select config from queues where key=$1", &[&queue_name])
        .map_err(iron_str_error));
    if config_row.is_empty() {
        return Ok(Response::with((status::NotFound, format!("No queue {}", queue_name))));
    }
    let config: types::QueueConfig = try!(serde_json::from_value(config_row.get(0).get("config"))
        .map_err(iron_str_error));
    let results = try!(conn.query(&format!("select id, task_name, state, hlc_tstamp from {}",
                        &queue_name),
               &[])
        .map_err(iron_str_error));
    let mut queue = Map::new();
    let now = clock::get_timestamp(req).time.as_timespec();
    let max_diff = Duration::milliseconds(config.timeout_ms);
    for row in &results {
        let id: Uuid = row.get("id");
        let mut state = try!(row_to_state(&row));
        if state == types::QueueState::Done {
            continue;
        }
        if state == types::QueueState::Working {
            let hlc_tstamp: Vec<u8> = row.get("hlc_tstamp");
            let when = try!(hybrid_clocks::Timestamp::read_bytes(Cursor::new(hlc_tstamp))
                .map_err(iron_str_error));
            let diff = now - when.time.as_timespec();
            if diff > max_diff {
                debug!("{} is out of date, so marking as pending", id);
                state = types::QueueState::Pending;
            }
        }
        let item = types::QueueListItem {
            task_name: row.get("task_name"),
            state: state,
        };
        queue.insert(id.to_string(),
                     serde_json::to_value(&item).map_err(iron_str_error)?);
    }
    let value = Value::Object(queue);
    Ok(Response::with((status::Ok, serde_json::ser::to_string(&value).unwrap())))
}

fn get_item_id(req: &mut Request) -> IronResult<Uuid> {
    Uuid::parse_str(&try!(get_req_key_with_iron_err(req, "id"))).map_err(iron_str_error)
}

fn get_queue_item(req: &mut Request) -> IronResult<Response> {
    let conn = get_db_connection!(&req);
    let queue_name = try!(get_queue_name(req));
    let id = try!(get_item_id(req));
    let results = try!(conn.query(&format!("select task_name, state, info, worker from {} where id=$1",
                        &queue_name),
               &[&id])
        .map_err(iron_str_error));
    if results.is_empty() {
        Ok(Response::with((status::NotFound, format!("No queue item {} in {}", id, queue_name))))
    } else {
        let row = results.get(0);
        let item = types::QueueItem {
            task_name: row.get("task_name"),
            state: try!(row_to_state(&row)),
            info: row.get("info"),
            worker: row.get("worker"),
        };
        Ok(Response::with((status::Ok, serde_json::to_string(&item).unwrap())))
    }
}

fn add_queue_item(req: &mut Request) -> IronResult<Response> {
    let mut json = try!(json_from_body(req));
    let queue_name = try!(get_queue_name(req));
    {
        let map = json.as_object_mut().unwrap();
        map.insert("queue_name".to_string(),
                   serde_json::to_value(&queue_name).map_err(iron_str_error)?);
    }
    let op = try!(serde_json::from_value::<types::QueueAdd>(json).map_err(iron_str_error));
    match add_queue_operation(QueueOperation::Add(op)) {
        Ok(val) => {
            let new_url = format!("http://{}:8000/queue/{}/{}",
                                  HOST.deref(),
                                  &queue_name,
                                  &val);
            Ok(Response::with((status::Created,
                               Redirect(iron::Url::parse(&new_url).expect("URL parsed ok")))))
        }
        Err(val) => Err(val),
    }
}

fn build_queue_progress(req: &mut Request) -> IronResult<types::QueueProgress> {
    let mut json = try!(json_from_body(req));
    {
        let queue_name = try!(get_queue_name(req));
        let id = try!(get_item_id(req));
        let map = json.as_object_mut().unwrap();
        map.insert("queue_name".to_string(),
                   serde_json::to_value(&queue_name).map_err(iron_str_error)?);
        map.insert("id".to_string(),
                   serde_json::to_value(&id).map_err(iron_str_error)?);
    }
    return Ok(try!(serde_json::from_value::<types::QueueProgress>(json).map_err(iron_str_error)));
}

fn progress_queue_item(req: &mut Request) -> IronResult<Response> {
    let op = try!(build_queue_progress(req));
    match add_queue_operation(QueueOperation::Progress(op)) {
        Ok(_) => get_queue_item(req),
        Err(val) => Err(val),
    }
}

fn finish_queue_item(req: &mut Request) -> IronResult<Response> {
    let op = try!(build_queue_progress(req));
    match add_queue_operation(QueueOperation::Done(op)) {
        Ok(_) => Ok(Response::with(status::Ok)),
        Err(val) => Err(val),
    }
}

fn make_queue_table(conn: &PostgresConnection) {
    conn.execute("CREATE TABLE IF NOT EXISTS queues (key VARCHAR(1024) PRIMARY KEY, config JSONB NOT NULL)",
                 &[])
        .expect("make queue table worked");
}

fn main() {
    log4rs::init_file("log.yaml", Default::default()).expect("log config ok");
    let client = hyper::client::Client::new();

    let mut map = serde_json::Map::new();
    map.insert("url".to_string(),
               serde_json::Value::String(format!("http://{}:8000/event", HOST.deref()).to_string()));
    let res = client.post(&format!("{}/register", SERVER_URL.deref()))
        .body(&serde_json::ser::to_string(&map).unwrap())
        .send()
        .expect("Register ok");
    assert_eq!(res.status, hyper::status::StatusCode::NoContent);

    let db_url: &str = &env::var("DATABASE_URL").expect("Needed DATABASE_URL");
    let pool = db::get_pool(db_url);
    let conn = pool.get().unwrap();
    make_queue_table(&conn);
    let (logger_before, logger_after) = Logger::new(None);
    let mut router = router::Router::new();
    router.post("/create", create_queue, "make a new queue");
    router.post("/event", new_event, "process incoming event");
    router.get("/queue/:queue_name", get_queue_items, "get items in queue");
    router.post("/queue/:queue_name", add_queue_item, "add a queue item");
    router.delete("/queue/:queue_name", delete_queue, "delete a queue");
    router.get("/queue/:queue_name/:id",
               get_queue_item,
               "get item from queue");
    router.put("/queue/:queue_name/:id",
               progress_queue_item,
               "mark progress on queue item");
    router.delete("/queue/:queue_name/:id",
                  finish_queue_item,
                  "mark queue item as finished");
    let mut chain = Chain::new(router);
    chain.link_before(logger_before);
    chain.link_after(logger_after);
    chain.link(PRead::<db::PostgresDB>::both(pool));
    let clock_state = clock::init_clock();
    chain.link_before(clock_state);
    info!("Pigtail booted");
    Iron::new(chain).http("0.0.0.0:8000").unwrap();
}
