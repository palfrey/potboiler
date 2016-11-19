#[macro_use]
extern crate log;
extern crate log4rs;
extern crate iron;
extern crate logger;
extern crate router;
extern crate persistent;
extern crate r2d2;
extern crate r2d2_postgres;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate potboiler_common;
extern crate serde_json;
extern crate hyper;
extern crate hybrid_clocks;
mod tables;

use iron::prelude::*;
use iron::status;
use logger::Logger;
use persistent::Read as PRead;
use persistent::State;
use potboiler_common::db;
use potboiler_common::server_id;
use potboiler_common::string_error::StringError;
use potboiler_common::types::{CRDT, LWW};
use r2d2_postgres::PostgresConnectionManager;
use router::Router;
use std::env;
use std::io::Read;
use std::ops::Deref;

pub type PostgresConnection = r2d2::PooledConnection<PostgresConnectionManager>;

lazy_static! {
    static ref SERVER_URL: String = env::var("SERVER_URL").expect("Needed SERVER_URL");
}

include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));

fn get_req_key<T: Into<String>>(req: &Request, key: T) -> Option<String> {
    req.extensions
        .get::<Router>()
        .unwrap()
        .find(&key.into())
        .map(|s| s.to_string())
}

fn get_key(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "get_key")))
}

fn update_key(req: &mut Request) -> IronResult<Response> {
    let body_string = {
        let mut body = String::new();
        req.body.read_to_string(&mut body).expect("could read from body");
        body
    };
    let mut json: serde_json::Value = match serde_json::de::from_str(&body_string) {
        Ok(val) => val,
        Err(err) => return Err(IronError::new(err, (status::BadRequest, "Bad JSON"))),
    };
    let map = json.as_object_mut().unwrap();
    map.insert("table".to_string(),
               serde_json::to_value(&get_req_key(req, "table").unwrap()));
    map.insert("key".to_string(),
               serde_json::to_value(&get_req_key(req, "key").unwrap()));

    let client = hyper::client::Client::new();

    let res = client.post(SERVER_URL.deref())
        .body(&serde_json::ser::to_string(&map).unwrap())
        .send()
        .expect("sender ok");
    assert_eq!(res.status, hyper::status::StatusCode::Created);
    Ok(Response::with((status::Ok, "update_key")))
}

fn make_table(conn: &PostgresConnection, table_name: &str) {
    conn.execute(&format!("CREATE TABLE IF NOT EXISTS {} (key VARCHAR(1024) PRIMARY KEY, value JSONB, \
                           crdt JSONB)",
                          &table_name),
                 &[])
        .expect("make table worked");
}

fn string_iron_error(error: &str) -> IronResult<Response> {
    Err(IronError::new(StringError(error.to_string()), (status::BadRequest, error)))
}

fn new_event(req: &mut Request) -> IronResult<Response> {
    let body_string = {
        let mut body = String::new();
        req.body.read_to_string(&mut body).expect("could read from body");
        body
    };
    let json: serde_json::Value = match serde_json::de::from_str(&body_string) {
        Ok(val) => val,
        Err(err) => return Err(IronError::new(err, (status::BadRequest, "Bad JSON"))),
    };
    info!("body: {:?}", json);
    let data = json.find("data").unwrap();
    let when: Timestamp<WallT> = serde_json::from_value(json.find("when").unwrap().clone()).unwrap();
    let change: Change = serde_json::from_value(data.clone()).unwrap();
    info!("change: {:?}", change);
    let tables = tables::get_tables(req);
    match tables.get(&change.table) {
        None => return string_iron_error("Can't find table"),
        Some(&val) => {
            match val {
                CRDT::LWW => {
                    match change.op {
                        Operation::Set => {
                            let conn = get_pg_connection!(&req);
                            let stmt =
                                conn.prepare(&format!("select crdt from {} where key=$1", &change.table))
                                    .expect("select prep worked");
                            let results = stmt.query(&[&change.key]).expect("bad query");
                            if results.is_empty() {
                                let lww = LWW {
                                    when: when,
                                    data: change.change.clone(),
                                };
                                conn.execute(&format!("INSERT INTO {} (key, value, crdt) VALUES ($1, $2, \
                                                       $3)",
                                                      &change.table),
                                             &[&change.key, &change.change, &serde_json::to_value(&lww)])
                                    .expect("insert worked");
                                if &change.table == tables::CONFIG_TABLE {
                                    make_table(&conn, &change.key);
                                    tables::add_table(req, &change.key, CRDT::LWW);
                                }
                            } else {
                                let row = results.get(0);
                                let raw_crdt: serde_json::Value = row.get("crdt");
                                let mut lww: LWW = serde_json::from_value(raw_crdt).expect("bad raw crdt");
                                if lww.when < when {
                                    lww.when = when;
                                    lww.data = change.change.clone();
                                    conn.execute(&format!("UPDATE {} set value=$2, crdt=$3 where key=$1",
                                                          &change.table),
                                                 &[&change.key, &change.change, &serde_json::to_value(&lww)])
                                        .expect("update worked");
                                } else {
                                    info!("Earlier event, skipping");
                                }
                            }
                        }
                        _ => {
                            return string_iron_error("LWW only supports Set, not Add/Remove");
                        }

                    }
                }
                _ => {
                    return string_iron_error("Only support LWW so far");
                }
            }
        }
    }
    Ok(Response::with(status::NoContent))
}

fn main() {
    log4rs::init_file("log.yaml", Default::default()).expect("log config ok");
    let client = hyper::client::Client::new();

    let mut map: serde_json::Map<String, String> = serde_json::Map::new();
    let host: &str = &env::var("HOST").unwrap_or("localhost".to_string());
    map.insert("url".to_string(),
               format!("http://{}:8001/kv/event", host).to_string());
    let res = client.post(&format!("{}/register", SERVER_URL.deref()))
        .body(&serde_json::ser::to_string(&map).unwrap())
        .send()
        .expect("Register ok");
    assert_eq!(res.status, hyper::status::StatusCode::NoContent);

    let db_url: &str = &env::var("DATABASE_URL").expect("Needed DATABASE_URL");
    let pool = db::get_pool(db_url);
    let conn = pool.get().unwrap();
    make_table(&conn, "_config");
    let (logger_before, logger_after) = Logger::new(None);
    let mut router = Router::new();
    router.get("/kv/:table/:key", get_key);
    router.post("/kv/:table/:key", update_key);
    router.post("/kv/event", new_event);
    let mut chain = Chain::new(router);
    chain.link_before(logger_before);
    chain.link_after(logger_after);
    chain.link_before(PRead::<server_id::ServerId>::one(server_id::setup()));
    chain.link(PRead::<db::PostgresDB>::both(pool));
    let tables = tables::init_tables(&conn);
    chain.link(State::<tables::Tables>::both(tables));
    info!("Potboiler-kv booted");
    Iron::new(chain).http("0.0.0.0:8001").unwrap();
}
