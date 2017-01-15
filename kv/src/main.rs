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
#[macro_use]
extern crate mime;
mod tables;

use iron::prelude::*;
use iron::status;
use logger::Logger;
use persistent::Read as PRead;
use persistent::State;
use potboiler_common::{db, iron_str_error, server_id};
use potboiler_common::string_error::StringError;
use potboiler_common::types::{CRDT, LWW, Log};
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
    {
        let map = json.as_object_mut().unwrap();
        map.insert("table".to_string(),
                   serde_json::to_value(&potboiler_common::get_req_key(req, "table").unwrap()));
        map.insert("key".to_string(),
                   serde_json::to_value(&potboiler_common::get_req_key(req, "key").unwrap()));
    }

    let change: Change = serde_json::from_value(json).map_err(iron_str_error)?;
    let string_change = serde_json::ser::to_string(&change).map_err(iron_str_error)?;
    match change.op {
        Operation::Add | Operation::Remove => {
            serde_json::from_value::<ORSetOp>(change.change).map_err(iron_str_error)?;
        }
        Operation::Create => {
            serde_json::from_value::<ORCreateOp>(change.change).map_err(iron_str_error)?;
        }
        Operation::Set => {
            if &change.table == tables::CONFIG_TABLE {
                serde_json::from_value::<LWWConfigOp>(change.change).map_err(iron_str_error)?;
            }
        }
    }
    let client = hyper::client::Client::new();
    let res = client.post(SERVER_URL.deref())
        .body(&string_change)
        .send()
        .expect("sender ok");
    assert_eq!(res.status, hyper::status::StatusCode::Created);
    Ok(Response::with((status::Ok, "update_key")))
}

fn make_table(conn: &PostgresConnection, table_name: &str, kind: &CRDT) -> IronResult<()> {
    match kind {
        &CRDT::LWW => {
            conn.execute(&format!("CREATE TABLE IF NOT EXISTS {} (key VARCHAR(1024) PRIMARY KEY, value \
                                   JSONB, crdt JSONB)",
                                  &table_name),
                         &[])
                .map_err(iron_str_error)?;
        }
        &CRDT::ORSET => {
            conn.execute(&format!("CREATE TABLE IF NOT EXISTS {} (key VARCHAR(1024) PRIMARY KEY, crdt \
                                   JSONB)",
                                  &table_name),
                         &[])
                .map_err(iron_str_error)?;
            conn.execute(&format!("CREATE TABLE IF NOT EXISTS {}_items (\
                                   collection VARCHAR(1024),
                                   key VARCHAR(1024), \
                                   item VARCHAR(1024), PRIMARY KEY(collection, key, item))",
                                  &table_name),
                         &[])
                .map_err(iron_str_error)?;
        }
        &CRDT::GSET => {
            error!("No G-Set make table yet");
        }
    }
    return Ok(());
}

fn raw_string_iron_error(error: &str) -> IronError {
    IronError::new(StringError(error.to_string()), (status::BadRequest, error))
}

fn string_iron_error(error: &str) -> IronResult<Response> {
    Err(raw_string_iron_error(error))
}

fn get_crdt(conn: &PostgresConnection,
            table: &String,
            key: &String)
            -> IronResult<Option<serde_json::Value>> {
    let stmt = conn.prepare(&format!("select crdt from {} where key=$1", table)).map_err(iron_str_error)?;
    let results = stmt.query(&[&key]).map_err(iron_str_error)?;
    if results.is_empty() {
        Ok(None)
    } else if results.len() == 1 {
        let row = results.get(0);
        let raw_crdt: serde_json::Value = row.get("crdt");
        Ok(Some(raw_crdt))
    } else {
        Err(iron_str_error(StringError::from(format!("{} entries for key {}", results.len(), key))))
    }
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
    let log = try!(serde_json::from_value::<Log>(json).map_err(iron_str_error));
    info!("log: {:?}", log);
    let change: Change = serde_json::from_value(log.data).unwrap();
    info!("change: {:?}", change);
    let tables = tables::get_tables(req);
    let table_type = match tables.get(&change.table) {
        None => return string_iron_error("Can't find table"),
        Some(&val) => val,
    };
    match table_type {
        CRDT::LWW => {
            match change.op {
                Operation::Set => {
                    let crdt_to_use: Option<CRDT> = if &change.table == tables::CONFIG_TABLE {
                        let config_op: LWWConfigOp =
                            serde_json::from_value(change.change.clone()).map_err(iron_str_error)?;
                        Some(config_op.crdt)
                    } else {
                        None
                    };
                    let conn = get_pg_connection!(&req);
                    let raw_crdt = get_crdt(&conn, &change.table, &change.key)?;
                    match raw_crdt {
                        None => {
                            let lww = LWW {
                                when: log.when,
                                data: change.change.clone(),
                            };
                            conn.execute(&format!("INSERT INTO {} (key, value, crdt) VALUES ($1, $2, $3)",
                                                  &change.table),
                                         &[&change.key, &change.change, &serde_json::to_value(&lww)])
                                .expect("insert worked");
                            if &change.table == tables::CONFIG_TABLE {
                                let crdt = crdt_to_use.unwrap();
                                make_table(&conn, &change.key, &crdt)?;
                                tables::add_table(req, &change.key, &crdt);
                            }
                        }
                        Some(raw_crdt) => {
                            let mut lww: LWW = serde_json::from_value(raw_crdt).expect("bad raw crdt");
                            if lww.when < log.when {
                                lww.when = log.when;
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
                }
                _ => {
                    return string_iron_error("LWW only supports Set, not Add/Remove");
                }
            }
        }
        CRDT::ORSET => {
            let op: Option<ORSetOp> = match change.op {
                Operation::Add | Operation::Remove => {
                    Some(serde_json::from_value(change.change).map_err(iron_str_error)?)
                }
                Operation::Create | Operation::Set => None,
            };
            let conn = get_pg_connection!(&req);
            let raw_crdt = get_crdt(&conn, &change.table, &change.key)?;
            let (mut crdt, existing) = match raw_crdt {
                Some(val) => (serde_json::from_value(val).map_err(iron_str_error)?, true),
                None => {
                    (ORSet {
                         adds: HashMap::new(),
                         removes: HashMap::new(),
                     },
                     false)
                }
            };
            let trans = conn.transaction().unwrap();
            match change.op {
                Operation::Add => {
                    let unwrap_op = op.unwrap();
                    if !crdt.removes.contains_key(&unwrap_op.key) && !crdt.adds.contains_key(&unwrap_op.key) {
                        trans.execute(&format!("INSERT INTO {}_items (collection, key, item) VALUES ($1, \
                                               $2, $3)",
                                              &change.table),
                                     &[&change.key, &unwrap_op.key, &unwrap_op.item])
                            .map_err(iron_str_error)?;
                        crdt.adds.insert(unwrap_op.key, unwrap_op.item);
                    }
                }
                Operation::Remove => {
                    let unwrap_op = op.unwrap();
                    trans.execute(&format!("DELETE FROM {}_items where collection=$1 and key=$2",
                                          &change.table),
                                 &[&change.key, &unwrap_op.key])
                        .map_err(iron_str_error)?;
                    crdt.adds.remove(&unwrap_op.key);
                    crdt.removes.insert(unwrap_op.key, unwrap_op.item);
                }
                Operation::Create => {
                    // Don't need to actually do anything to the item lists
                }
                _ => {
                    return string_iron_error("ORSET only supports Add/Create/Remove");
                }
            }
            debug!("OR-Set for {}: {:?}", &change.table, &crdt);
            if existing {
                trans.execute(&format!("UPDATE {} set crdt=$2 where key=$1", &change.table),
                             &[&change.key, &serde_json::to_value(&crdt)])
                    .map_err(iron_str_error)?;
            } else {
                trans.execute(&format!("INSERT INTO {} (key, crdt) VALUES ($1, $2)", &change.table),
                             &[&change.key, &serde_json::to_value(&crdt)])
                    .map_err(iron_str_error)?;
            }
            let mut items: Vec<&str> = Vec::new();
            for key in crdt.adds.keys() {
                if crdt.removes.contains_key(key) {
                    continue;
                }
                items.push(crdt.adds.get(key).unwrap());
            }
            debug!("Items: {:?}", items);
            trans.commit().map_err(iron_str_error)?;
        }
        _ => {
            return string_iron_error("Only support LWW and OR-Set so far");
        }
    }
    Ok(Response::with(status::NoContent))
}

fn list_tables(req: &mut Request) -> IronResult<Response> {
    let tables = tables::get_tables(req);
    let mut table_names = vec![];
    for t in tables.keys() {
        table_names.push(t);
    }
    Ok(Response::with((status::Ok, mime!(Application / Json), serde_json::to_string(&table_names).unwrap())))
}

fn list_keys(req: &mut Request) -> IronResult<Response> {
    let table = potboiler_common::get_req_key(req, "table").ok_or(raw_string_iron_error("No table key"))?;
    let mut key_names = vec![];
    let conn = get_pg_connection!(&req);
    let stmt = conn.prepare(&format!("select key from {}", table)).map_err(iron_str_error)?;
    for row in &stmt.query(&[]).map_err(iron_str_error)? {
        let key: String = row.get("key");
        key_names.push(key);
    }
    Ok(Response::with((status::Ok,
                       mime!(Application / Json),
                       serde_json::to_string(&key_names).map_err(iron_str_error)?)))
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
    match make_table(&conn, tables::CONFIG_TABLE, &CRDT::LWW) {
        Ok(_) => {}
        Err(err) => {
            error!("Error while making config table: {}", err);
            return;
        }
    }
    let (logger_before, logger_after) = Logger::new(None);
    let mut router = Router::new();
    router.get("/kv", list_tables);
    router.get("/kv/:table", list_keys);
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
