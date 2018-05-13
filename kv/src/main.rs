#[macro_use]
extern crate log;
extern crate log4rs;
extern crate iron;
extern crate logger;
extern crate router;
extern crate persistent;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
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

#[macro_use]
extern crate error_chain;

extern crate serde;
#[macro_use]
extern crate serde_derive;
mod serde_types;

use iron::prelude::*;
use iron::status;
use logger::Logger;
use persistent::Read as PRead;
use persistent::State;
use potboiler_common::{db, pg, server_id};
use potboiler_common::types::{CRDT, Log};
use router::Router;
use serde_types::*;
use std::collections::HashMap;
use std::env;
use std::io::Read;
use std::ops::Deref;

error_chain! {
    errors {
        WrongResultsCount(key: String, count: usize)
        NoTableKey
        UnsupportedCRDT(name: CRDT)
        UnsupportedORSETOp(name: Operation)
        UnsupportedLWWOp(name: Operation)
        NoTable(name: String)
    }
    links {
        DbError(db::Error, db::ErrorKind);
    }
    foreign_links {
        SerdeError(serde_json::Error);
    }
}

iron_error_from!();

lazy_static! {
    static ref SERVER_URL: String = env::var("SERVER_URL").expect("Needed SERVER_URL");
}

fn get_key(req: &mut Request) -> IronResult<Response> {
    let conn = get_db_connection!(&req);
    let mut items = Vec::new();
    for row in conn.query(&format!("SELECT key, item, metadata FROM {}_items where collection={}", 
        &potboiler_common::get_req_key(req, "table").unwrap(),
        &potboiler_common::get_req_key(req, "key").unwrap())).unwrap().iter() {
        let key: String = row.get("key");
        let item: String = row.get("item");
        let metadata: serde_json::Value = row.get("metadata");
        items.push(ORSetOp {
            item: item,
            key: key,
            metadata: metadata,
        });
    }
    Ok(Response::with((status::Ok, serde_json::ser::to_string(&items).unwrap())))
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
                   serde_json::to_value(
                       &potboiler_common::get_req_key(req, "table").unwrap()).unwrap());
        map.insert("key".to_string(),
                   serde_json::to_value(
                       &potboiler_common::get_req_key(req, "key").unwrap()).unwrap());
    }

    let change: Change = serde_json::from_value(json).map_err(|e| Error::from(e))?;
    let string_change = serde_json::ser::to_string(&change).map_err(|e| Error::from(e))?;
    match change.op {
        Operation::Add | Operation::Remove => {
            serde_json::from_value::<ORSetOp>(change.change).map_err(|e| Error::from(e))?;
        }
        Operation::Create => {
            serde_json::from_value::<ORCreateOp>(change.change).map_err(|e| Error::from(e))?;
        }
        Operation::Set => {
            if &change.table == tables::CONFIG_TABLE {
                serde_json::from_value::<LWWConfigOp>(change.change).map_err(|e| Error::from(e))?;
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

fn make_table(conn: &db::Connection, table_name: &str, kind: &CRDT) -> IronResult<()> {
    match kind {
        &CRDT::LWW => {
            conn.execute(&format!("CREATE TABLE IF NOT EXISTS {} (key VARCHAR(1024) PRIMARY KEY, value \
                                   JSONB NOT NULL, crdt JSONB NOT NULL)",
                                  &table_name)).map_err(|e| Error::from(e))?;
        }
        &CRDT::ORSET => {
            conn.execute(&format!("CREATE TABLE IF NOT EXISTS {} (key VARCHAR(1024) PRIMARY KEY, crdt \
                                   JSONB NOT NULL)",
                                  &table_name)).map_err(|e| Error::from(e))?;
            conn.execute(&format!("CREATE TABLE IF NOT EXISTS {}_items (\
                                   collection VARCHAR(1024) NOT NULL,
                                   key VARCHAR(1024) NOT NULL, \
                                   item VARCHAR(1024) NOT NULL, \
                                   metadata JSONB NOT NULL, \
                                   PRIMARY KEY(collection, key, item))",
                                  &table_name)).map_err(|e| Error::from(e))?;
        }
        &CRDT::GSET => {
            error!("No G-Set make table yet");
        }
    }
    return Ok(());
}

fn get_crdt(conn: &db::Connection,
            table: &String,
            key: &String)
            -> IronResult<Option<serde_json::Value>> {
    let results = conn.query(&format!("select crdt from {} where key='{}'", table, &key)).map_err(|e| Error::from(e))?;
    if results.is_empty() {
        Ok(None)
    } else if results.len() == 1 {
        let row = results.get(0);
        let raw_crdt: serde_json::Value = row.get("crdt");
        Ok(Some(raw_crdt))
    } else {
        Err(ErrorKind::WrongResultsCount(key.to_string(), results.len()).into())
    }
}

fn new_event(req: &mut Request) -> IronResult<Response> {
    let body_string = {
        let mut body = String::new();
        req.body.read_to_string(&mut body).expect("could read from body");
        body
    };
    let json: serde_json::Value = serde_json::de::from_str(&body_string).map_err(|e| Error::from(e))?;
    info!("body: {:?}", json);
    let log = serde_json::from_value::<Log>(json).map_err(|e| Error::from(e))?;
    info!("log: {:?}", log);
    let change: Change = serde_json::from_value(log.data).unwrap();
    info!("change: {:?}", change);
    let tables = tables::get_tables(req);
    let table_type = match tables.get(&change.table) {
        None => bail!(ErrorKind::NoTable(change.table)),
        Some(&val) => val,
    };
    match table_type {
        CRDT::LWW => {
            match change.op {
                Operation::Set => {
                    let crdt_to_use: Option<CRDT> = if &change.table == tables::CONFIG_TABLE {
                        let config_op: LWWConfigOp =
                            serde_json::from_value(change.change.clone()).map_err(|e| Error::from(e))?;
                        Some(config_op.crdt)
                    } else {
                        None
                    };
                    let conn = get_db_connection!(&req);
                    let raw_crdt = get_crdt(&conn, &change.table, &change.key)?;
                    match raw_crdt {
                        None => {
                            let lww = LWW {
                                when: log.when,
                                data: change.change.clone(),
                            };
                            conn.execute(&format!("INSERT INTO {} (key, value, crdt) VALUES ('{}','{}','{}')",
                                                  &change.table,
                                                  &change.key,
                                                  &change.change,
                                                  &serde_json::to_value(&lww).map_err(|e| Error::from(e))?))
                                .map_err(|e| Error::from(e))?;
                            if &change.table == tables::CONFIG_TABLE {
                                let crdt = crdt_to_use.unwrap();
                                make_table(&conn, &change.key, &crdt)?;
                                tables::add_table(req, &change.key, &crdt);
                            }
                        }
                        Some(raw_crdt) => {
                            let mut lww: LWW = serde_json::from_value(raw_crdt).map_err(|e| Error::from(e))?;
                            if lww.when < log.when {
                                lww.when = log.when;
                                lww.data = change.change.clone();
                                conn.execute(&format!("UPDATE {} set value='{}', crdt='{}' where key='{}'",
                                                      &change.table,
                                                      &change.change,
                                                      &change.key,
                                                      &serde_json::to_value(&lww).map_err(|e| Error::from(e))?))
                                                      .map_err(|e| Error::from(e))?;
                            } else {
                                info!("Earlier event, skipping");
                            }
                        }
                    }
                }
                _ => {
                    return Err(ErrorKind::UnsupportedLWWOp(change.op).into());
                }
            }
        }
        CRDT::ORSET => {
            let op: Option<ORSetOp> = match change.op {
                Operation::Add | Operation::Remove => {
                    Some(serde_json::from_value(change.change).map_err(|e| Error::from(e))?)
                }
                Operation::Create | Operation::Set => None,
            };
            let conn = get_db_connection!(&req);
            let raw_crdt = get_crdt(&conn, &change.table, &change.key)?;
            let (mut crdt, existing) = match raw_crdt {
                Some(val) => (serde_json::from_value(val).map_err(|e| Error::from(e))?, true),
                None => {
                    (ORSet {
                         adds: HashMap::new(),
                         removes: HashMap::new(),
                     },
                     false)
                }
            };
            //let trans = conn.transaction().unwrap();
            let trans = conn;
            match change.op {
                Operation::Add => {
                    let unwrap_op = op.unwrap();
                    if !crdt.removes.contains_key(&unwrap_op.key) {
                        let metadata = unwrap_op.metadata;
                        if crdt.adds.contains_key(&unwrap_op.key) {
                            trans.execute(&format!("UPDATE {}_items set metadata='{}' where collection='{}' \
                                                   and key='{}'",
                                                  &change.table, &metadata, &change.key, &unwrap_op.key))
                                .map_err(|e| Error::from(e))?;
                        } else {
                            trans.execute(&format!("INSERT INTO {}_items (collection, key, item, metadata) \
                                                   VALUES ('{}', '{}', '{}', '{}')",
                                                  &change.table, &change.key, &unwrap_op.key, &unwrap_op.item, &metadata))
                                .map_err(|e| Error::from(e))?;
                            crdt.adds.insert(unwrap_op.key, unwrap_op.item);
                        }
                    }
                }
                Operation::Remove => {
                    let unwrap_op = op.unwrap();
                    trans.execute(&format!("DELETE FROM {}_items where collection='{}' and key='{}'",
                                          &change.table, &change.key, &unwrap_op.key))
                        .map_err(|e| Error::from(e))?;
                    crdt.adds.remove(&unwrap_op.key);
                    crdt.removes.insert(unwrap_op.key, unwrap_op.item);
                }
                Operation::Create => {
                    // Don't need to actually do anything to the item lists
                }
                _ => {
                    return Err(ErrorKind::UnsupportedORSETOp(change.op).into());
                }
            }
            debug!("OR-Set for {}: {:?}", &change.table, &crdt);
            if existing {
                trans.execute(&format!("UPDATE {} set crdt='{}' where key='{}'", 
                    &change.table, &serde_json::to_value(&crdt).map_err(|e| ErrorKind::SerdeError(e))?, &change.key))
                    .map_err(|e| Error::from(e))?;
            } else {
                trans.execute(&format!("INSERT INTO {} (key, crdt) VALUES ('{}', '{}')", 
                    &change.table, &serde_json::to_value(&crdt).map_err(|e| ErrorKind::SerdeError(e))?, &change.key))
                    .map_err(|e| Error::from(e))?;
            }
            let mut items: Vec<&str> = Vec::new();
            for key in crdt.adds.keys() {
                if crdt.removes.contains_key(key) {
                    continue;
                }
                items.push(crdt.adds.get(key).unwrap());
            }
            debug!("Items: {:?}", items);
            //trans.commit()?;
        }
        _ => {
            return Err(ErrorKind::UnsupportedCRDT(table_type).into());
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
    let table = potboiler_common::get_req_key(req, "table").ok_or(ErrorKind::NoTableKey)?;
    let conn = get_db_connection!(&req);
    let mut key_names = vec![];
    for row in &conn.query(&format!("select key from {}", table)).map_err(|e| Error::from(e))? {
        let key: String = row.get("key");
        key_names.push(key);
    }
    Ok(Response::with((status::Ok,
                       mime!(Application / Json),
                       serde_json::to_string(&key_names).map_err(|e| ErrorKind::SerdeError(e))?)))
}

fn main() {
    log4rs::init_file("log.yaml", Default::default()).expect("log config ok");
    let client = hyper::client::Client::new();

    let mut map = serde_json::Map::new();
    let host: &str = &env::var("HOST").unwrap_or("localhost".to_string());
    map.insert("url".to_string(),
               serde_json::Value::String(format!("http://{}:8001/kv/event", host)));
    let res = client.post(&format!("{}/register", SERVER_URL.deref()))
        .body(&serde_json::ser::to_string(&map).unwrap())
        .send()
        .expect("Register ok");
    assert_eq!(res.status, hyper::status::StatusCode::NoContent);

    let db_url: &str = &env::var("DATABASE_URL").expect("Needed DATABASE_URL");
    let pool = pg::get_pool(db_url).unwrap();
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
    router.get("/kv", list_tables, "list tables");
    router.get("/kv/:table", list_keys, "list keys for a table");
    router.get("/kv/:table/:key", get_key, "get a key");
    router.post("/kv/:table/:key", update_key, "update a key");
    router.post("/kv/event", new_event, "process new event");
    let mut chain = Chain::new(router);
    chain.link_before(logger_before);
    chain.link_after(logger_after);
    chain.link_before(PRead::<server_id::ServerId>::one(server_id::setup()));
    chain.link(PRead::<db::PoolKey>::both(pool));
    let tables = tables::init_tables(&conn);
    chain.link(State::<tables::Tables>::both(tables));
    info!("Potboiler-kv booted");
    Iron::new(chain).http("0.0.0.0:8001").unwrap();
}
