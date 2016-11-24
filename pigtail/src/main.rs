#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate r2d2;
extern crate r2d2_postgres;
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

use iron::modifiers::Redirect;
use iron::prelude::{Chain, Iron, IronError, IronResult, Request, Response};
use iron::status;
use logger::Logger;
use persistent::Read as PRead;
use potboiler_common::db;
use potboiler_common::string_error::StringError;
use potboiler_common::types::Log;
use serde_json::{Map, Value};
use std::env;
use std::ops::Deref;
use types::QueueOperation;
use uuid::Uuid;

mod types;

pub type PostgresConnection = r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>;

lazy_static! {
    static ref SERVER_URL: String = env::var("SERVER_URL").expect("Needed SERVER_URL");
    static ref HOST: String = env::var("HOST").expect("Needed HOST");
}

fn iron_str_error<T: std::error::Error + std::marker::Send + 'static>(se: T) -> iron::IronError {
    let desc = format!("{:?}", se);
    return IronError::new(se, (status::BadRequest, desc));
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

fn new_event(req: &mut Request) -> IronResult<Response> {
    let json = try!(json_from_body(req));
    let log = try!(serde_json::from_value::<Log>(json).map_err(iron_str_error));
    info!("body: {:?}", log);
    let op = try!(serde_json::from_value::<QueueOperation>(log.data).map_err(iron_str_error));
    info!("op: {:?}", op);
    let conn = get_pg_connection!(&req);
    match op {
        QueueOperation::Create(create) => {
            info!("create: {:?}", create);
            let trans = conn.transaction().unwrap();
            let qc = types::QueueConfig { timeout_ms: create.timeout_ms };
            trans.execute("INSERT INTO queues (key, config) VALUES($1, $2)",
                         &[&create.name, &serde_json::to_value(&qc)])
                .unwrap();
            trans.execute(&format!("CREATE TABLE IF NOT EXISTS {} (id UUID PRIMARY KEY, task_name \
                                   VARCHAR(2083), state VARCHAR(8), info JSONB)",
                                  &create.name),
                         &[])
                .expect("make particular queue table worked");
            trans.commit().unwrap();
        }
        QueueOperation::Add(add) => {
            info!("add: {:?}", add);
            conn.execute(&format!("INSERT INTO {} (id, task_name, state, info) VALUES($1, $2, $3, $4)",
                                  add.queue_name),
                         &[&log.id,
                           &add.task_name,
                           &String::from("pending"),
                           &serde_json::to_value(&add.data)])
                .unwrap();
        }
        _ => {}
    };
    Ok(Response::with(status::NoContent))
}

fn get_queue_name(req: &mut Request) -> IronResult<String> {
    Ok(try!(potboiler_common::get_req_key(req, "queue_name")
        .ok_or(iron_str_error(StringError::from("No queue_name")))))
}

fn get_queue_items(req: &mut Request) -> IronResult<Response> {
    let conn = get_pg_connection!(&req);
    let queue_name = try!(get_queue_name(req));
    let results = try!(conn.query(&format!("select id, task_name, state from {}", &queue_name),
               &[])
        .map_err(iron_str_error));
    let mut queue = Map::new();
    for row in &results {
        let id: Uuid = row.get("id");
        let item = types::QueueListItem {
            task_name: row.get("task_name"),
            state: serde_json::from_str(&row.get::<&str, String>("state")).unwrap(),
        };
        queue.insert(id.to_string(), serde_json::to_value(&item));
    }
    let value = Value::Object(queue);
    Ok(Response::with((status::Ok, serde_json::ser::to_string(&value).unwrap())))
}

fn add_queue_item(req: &mut Request) -> IronResult<Response> {
    let mut json = try!(json_from_body(req));
    {
        let queue_name = try!(get_queue_name(req));
        let map = json.as_object_mut().unwrap();
        map.insert("queue_name".to_string(), serde_json::to_value(&queue_name));
    }
    let op = try!(serde_json::from_value::<types::QueueAdd>(json).map_err(iron_str_error));
    match add_queue_operation(QueueOperation::Add(op)) {
        Ok(_) => Ok(Response::with(status::Created)),
        Err(val) => Err(val),
    }
}

fn make_queue_table(conn: &PostgresConnection) {
    conn.execute("CREATE TABLE IF NOT EXISTS queues (key VARCHAR(1024) PRIMARY KEY, config JSONB)",
                 &[])
        .expect("make queue table worked");
}

fn main() {
    log4rs::init_file("log.yaml", Default::default()).expect("log config ok");
    let client = hyper::client::Client::new();

    let mut map: serde_json::Map<String, String> = serde_json::Map::new();
    map.insert("url".to_string(),
               format!("http://{}:8000/event", HOST.deref()).to_string());
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
    router.post("/create", create_queue);
    router.post("/event", new_event);
    router.get("/queue/:queue_name", get_queue_items);
    router.post("/queue/:queue_name", add_queue_item);
    let mut chain = Chain::new(router);
    chain.link_before(logger_before);
    chain.link_after(logger_after);
    chain.link(PRead::<db::PostgresDB>::both(pool));
    info!("Pigtail booted");
    Iron::new(chain).http("0.0.0.0:8000").unwrap();
}
