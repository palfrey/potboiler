#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    warnings,
    trivial_numeric_casts,
    unstable_features,
    unused,
    future_incompatible
)]

#[macro_use]
extern crate log;
use iron;
use log4rs;
use logger;
use persistent;
use router;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate potboiler_common;
use hybrid_clocks;
use hyper;
use serde_json;
#[macro_use]
extern crate mime;
use r2d2;
mod tables;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate serde_derive;
mod serde_types;

#[cfg(test)]
#[macro_use]
extern crate yup_hyper_mock as hyper_mock;

use crate::serde_types::*;
use iron::prelude::*;
use iron::status;
use logger::Logger;
use persistent::Read as PRead;
use persistent::State;
use potboiler_common::types::{Log, CRDT};
use potboiler_common::{db, http_client, pg, server_id};
use router::Router;
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
        ConfigTableCreation
    }
    links {
        DbError(db::Error, db::ErrorKind);
    }
    foreign_links {
        Serde(serde_json::Error);
        Hyper(hyper::Error);
        Log(log4rs::Error);
        R2D2(r2d2::Error);
    }
}

iron_error_from!();

lazy_static! {
    static ref SERVER_URL: String = env::var("SERVER_URL").expect("Needed SERVER_URL");
}

fn get_table_kind(table: &str) -> CRDT {
    if table == tables::CONFIG_TABLE {
        return CRDT::LWW;
    } else {
        //let tables = tables::get_tables(req);
        unimplemented!()
    }
}

fn get_key(req: &mut Request) -> IronResult<Response> {
    let conn = get_db_connection!(&req);
    let table = &potboiler_common::get_req_key(req, "table").unwrap();
    let key = &potboiler_common::get_req_key(req, "key").unwrap();
    let kind = get_table_kind(table);
    match kind {
        CRDT::ORSET => {
            match conn.query(&format!(
                "SELECT key, item, metadata FROM {}_items where collection={}",
                table, key
            )) {
                Ok(rows) => {
                    let mut items = Vec::new();
                    for row in rows.iter() {
                        let key: String = row.get("key");
                        let item: String = row.get("item");
                        let metadata: serde_json::Value = row.get("metadata");
                        items.push(ORSetOp {
                            item: item,
                            key: key,
                            metadata: metadata,
                        });
                    }
                    return Ok(Response::with((
                        status::Ok,
                        serde_json::ser::to_string(&items).unwrap(),
                    )));
                }
                Err(db::Error(db::ErrorKind::NoSuchTable, _)) => {
                    return Ok(Response::with((status::NotFound, format!("No such table {}", table))));
                }
                Err(err) => {
                    error!("Error while getting key {} from table {}: {:?}", key, table, err);
                    return Ok(Response::with(status::InternalServerError));
                }
            }
        }
        CRDT::LWW => match conn.query(&format!("SELECT item FROM {} where key={}", table, key)) {
            Ok(rows) => {
                for row in rows.iter() {
                    let item: String = row.get("item");
                    return Ok(Response::with((status::Ok, item)));
                }
                return Ok(Response::with(status::NotFound));
            }
            Err(db::Error(db::ErrorKind::NoSuchTable, _)) => {
                return Ok(Response::with((status::NotFound, format!("No such table {}", table))));
            }
            Err(err) => {
                error!("Error while getting key {} from table {}: {:?}", key, table, err);
                return Ok(Response::with(status::InternalServerError));
            }
        },
        _ => {
            return Ok(Response::with((
                status::InternalServerError,
                format!("No key getter for {:?}", kind),
            )));
        }
    }
}

fn update_key(req: &mut Request) -> IronResult<Response> {
    let body_string = {
        let mut body = String::new();
        req.body.read_to_string(&mut body).expect("could read from body");
        body
    };
    let mut json: serde_json::Value = match serde_json::de::from_str(&body_string) {
        Ok(val) => val,
        Err(err) => {
            info!("Failed to parse '{}'", &body_string);
            return Err(IronError::new(err, (status::BadRequest, "Bad JSON")));
        }
    };
    {
        let map = json.as_object_mut().unwrap();
        map.insert(
            "table".to_string(),
            serde_json::to_value(&potboiler_common::get_req_key(req, "table").unwrap()).unwrap(),
        );
        map.insert(
            "key".to_string(),
            serde_json::to_value(&potboiler_common::get_req_key(req, "key").unwrap()).unwrap(),
        );
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
    let client = get_http_client!(req);
    let res = client
        .post(SERVER_URL.deref())
        .body(&string_change)
        .send()
        .expect("sender ok");
    assert_eq!(res.status, hyper::status::StatusCode::Created);
    Ok(Response::with(status::Ok))
}

fn make_table(conn: &db::Connection, table_name: &str, kind: &CRDT) -> Result<()> {
    match kind {
        &CRDT::LWW => {
            conn.execute(&format!(
                "CREATE TABLE IF NOT EXISTS {} (key VARCHAR(1024) PRIMARY KEY, value \
                 JSONB NOT NULL, crdt JSONB NOT NULL)",
                &table_name
            ))
            .map_err(|e| Error::from(e))?;
        }
        &CRDT::ORSET => {
            conn.execute(&format!(
                "CREATE TABLE IF NOT EXISTS {} (key VARCHAR(1024) PRIMARY KEY, crdt \
                 JSONB NOT NULL)",
                &table_name
            ))
            .map_err(|e| Error::from(e))?;
            conn.execute(&format!(
                "CREATE TABLE IF NOT EXISTS {}_items (\
                                   collection VARCHAR(1024) NOT NULL,
                                   key VARCHAR(1024) NOT NULL, \
                                   item VARCHAR(1024) NOT NULL, \
                                   metadata JSONB NOT NULL, \
                                   PRIMARY KEY(collection, key, item))",
                &table_name
            ))
            .map_err(|e| Error::from(e))?;
        }
        &CRDT::GSET => {
            error!("No G-Set make table yet");
        }
    }
    return Ok(());
}

fn get_crdt(conn: &db::Connection, table: &String, key: &String) -> IronResult<Option<serde_json::Value>> {
    let results = conn
        .query(&format!("select crdt from {} where key='{}'", table, &key))
        .map_err(|e| Error::from(e))?;
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
        CRDT::LWW => match change.op {
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
                        conn.execute(&format!(
                            "INSERT INTO {} (key, value, crdt) VALUES ('{}','{}','{}')",
                            &change.table,
                            &change.key,
                            &change.change,
                            &serde_json::to_value(&lww).map_err(|e| Error::from(e))?
                        ))
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
                            conn.execute(&format!(
                                "UPDATE {} set value='{}', crdt='{}' where key='{}'",
                                &change.table,
                                &change.change,
                                &change.key,
                                &serde_json::to_value(&lww).map_err(|e| Error::from(e))?
                            ))
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
        },
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
                None => (
                    ORSet {
                        adds: HashMap::new(),
                        removes: HashMap::new(),
                    },
                    false,
                ),
            };
            //let trans = conn.transaction().unwrap();
            let trans = conn;
            match change.op {
                Operation::Add => {
                    let unwrap_op = op.unwrap();
                    if !crdt.removes.contains_key(&unwrap_op.key) {
                        let metadata = unwrap_op.metadata;
                        if crdt.adds.contains_key(&unwrap_op.key) {
                            trans
                                .execute(&format!(
                                    "UPDATE {}_items set metadata='{}' where collection='{}' \
                                     and key='{}'",
                                    &change.table, &metadata, &change.key, &unwrap_op.key
                                ))
                                .map_err(|e| Error::from(e))?;
                        } else {
                            trans
                                .execute(&format!(
                                    "INSERT INTO {}_items (collection, key, item, metadata) \
                                     VALUES ('{}', '{}', '{}', '{}')",
                                    &change.table, &change.key, &unwrap_op.key, &unwrap_op.item, &metadata
                                ))
                                .map_err(|e| Error::from(e))?;
                            crdt.adds.insert(unwrap_op.key, unwrap_op.item);
                        }
                    }
                }
                Operation::Remove => {
                    let unwrap_op = op.unwrap();
                    trans
                        .execute(&format!(
                            "DELETE FROM {}_items where collection='{}' and key='{}'",
                            &change.table, &change.key, &unwrap_op.key
                        ))
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
                trans
                    .execute(&format!(
                        "UPDATE {} set crdt='{}' where key='{}'",
                        &change.table,
                        &serde_json::to_value(&crdt).map_err(|e| ErrorKind::Serde(e))?,
                        &change.key
                    ))
                    .map_err(|e| Error::from(e))?;
            } else {
                trans
                    .execute(&format!(
                        "INSERT INTO {} (key, crdt) VALUES ('{}', '{}')",
                        &change.table,
                        &serde_json::to_value(&crdt).map_err(|e| ErrorKind::Serde(e))?,
                        &change.key
                    ))
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
    Ok(Response::with((
        status::Ok,
        mime!(Application / Json),
        serde_json::to_string(&table_names).unwrap(),
    )))
}

fn list_keys(req: &mut Request) -> IronResult<Response> {
    let table = potboiler_common::get_req_key(req, "table").ok_or(ErrorKind::NoTableKey)?;
    let conn = get_db_connection!(&req);
    let mut key_names = vec![];
    for row in &conn
        .query(&format!("select key from {}", table))
        .map_err(|e| Error::from(e))?
    {
        let key: String = row.get("key");
        key_names.push(key);
    }
    Ok(Response::with((
        status::Ok,
        mime!(Application / Json),
        serde_json::to_string(&key_names).map_err(|e| ErrorKind::Serde(e))?,
    )))
}

fn app_router(pool: db::Pool) -> Result<iron::Chain> {
    let conn = pool.get().unwrap();
    match make_table(&conn, tables::CONFIG_TABLE, &CRDT::LWW) {
        Ok(_) => {}
        Err(err) => {
            bail!(Error::with_chain(err, ErrorKind::ConfigTableCreation));
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
    chain.link(PRead::<db::PoolKey>::both(pool));
    let tables = tables::init_tables(&conn);
    chain.link(State::<tables::Tables>::both(tables));
    Ok(chain)
}

fn register(client: &hyper::Client) -> Result<()> {
    let mut map = serde_json::Map::new();
    let host: &str = &env::var("HOST").unwrap_or("localhost".to_string());
    map.insert(
        "url".to_string(),
        serde_json::Value::String(format!("http://{}:8001/kv/event", host)),
    );
    let res = client
        .post(&format!("{}/register", SERVER_URL.deref()))
        .body(&serde_json::ser::to_string(&map)?)
        .send()?;
    assert_eq!(res.status, hyper::status::StatusCode::NoContent);
    Ok(())
}

quick_main!(|| -> Result<()> {
    log4rs::init_file("log.yaml", Default::default())?;
    let db_url: &str = &env::var("DATABASE_URL").expect("Needed DATABASE_URL");
    let pool = pg::get_pool(db_url)?;
    let mut router = app_router(pool)?;
    router.link_before(PRead::<server_id::ServerId>::one(server_id::setup()));
    let client = hyper::client::Client::new();
    register(&client)?;
    http_client::set_client(&mut router, client);
    info!("Potboiler-kv booted");
    Iron::new(router).http("0.0.0.0:8001")?;
    Ok(())
});

#[cfg(test)]
mod test {
    use iron_test::request;
    use iron_test::response::extract_body_to_string;

    use crate::app_router;
    use crate::db;
    use crate::http_client;
    use crate::register;
    use hyper;
    use iron;
    use iron::status::Status;
    use iron::Headers;
    use log4rs;

    fn setup_logging() {
        let stdout = log4rs::append::console::ConsoleAppender::builder().build();
        let config = log4rs::config::Config::builder()
            .appender(log4rs::config::Appender::builder().build("stdout", Box::new(stdout)))
            .build(
                log4rs::config::Root::builder()
                    .appender("stdout")
                    .build(log::LevelFilter::Debug),
            )
            .unwrap();
        log4rs::init_config(config).unwrap();
    }

    fn test_get_route(router: &iron::Chain, path: &str, expected_body: &str, expected_status: Status) {
        let response = request::get(&format!("http://localhost:8001/{}", path), Headers::new(), router).unwrap();
        assert_eq!(response.status.unwrap(), expected_status);
        let result = extract_body_to_string(response);
        assert_eq!(result, expected_body);
    }

    fn test_post_route(router: &iron::Chain, path: &str, body: &str, expected_body: &str, expected_status: Status) {
        let resp = request::post(&format!("http://localhost:8001/{}", path), Headers::new(), body, router);
        let response = match resp {
            Ok(response) => response,
            Err(err) => err.response,
        };
        assert_eq!(response.status.unwrap(), expected_status);
        let result = extract_body_to_string(response);
        assert_eq!(result, expected_body);
    }

    fn setup_db(tables: Vec<db::TestRow>) -> db::TestConnection {
        let mut conn = super::db::TestConnection::new();
        conn.add_test_execute(
            concat!(
                r"CREATE TABLE IF NOT EXISTS _config \(key VARCHAR\(1024\) PRIMARY KEY, ",
                r"value JSONB NOT NULL, crdt JSONB NOT NULL\)"
            ),
            0,
        );
        conn.add_test_query("select key, value from _config", tables);
        return conn;
    }

    fn setup_router(conn: db::TestConnection) -> iron::Chain {
        super::env::set_var("SERVER_URL", "http://core");
        let pool = super::db::Pool::TestPool(conn);
        app_router(pool).unwrap()
    }

    #[test]
    fn no_key_table() {
        let mut conn = setup_db(vec![]);
        conn.add_test_query("SELECT item FROM _config where key=test", vec![]);
        let router = setup_router(conn);
        test_get_route(&router, "kv/_config/test", "", Status::NotFound);
    }

    #[test]
    fn key_table() {
        setup_logging();
        let conn = setup_db(vec![]);
        let mut router = setup_router(conn);
        mock_connector_in_order!(MockCore {
            "HTTP/1.1 204 NoContent\r\n\r\n" // register
            "HTTP/1.1 201 Created\r\n\r\n" // add key
        });
        let client = hyper::Client::with_connector(MockCore::default());
        register(&client).unwrap();
        http_client::set_client(&mut router, client);
        test_post_route(
            &router,
            "kv/_config/test",
            r#"{"op":"set","change":{"crdt": "LWW"}}"#,
            "",
            Status::Ok,
        );
    }
}
