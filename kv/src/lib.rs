#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    warnings,
    trivial_numeric_casts,
    unstable_features,
    unused,
    future_incompatible
)]

use crate::serde_types::*;
use actix_web::{
    http::{Method, StatusCode},
    App, HttpResponse, Json, Path, ResponseError, State,
};
use failure::{bail, Error, Fail};
use lazy_static::lazy_static;
use log::{debug, error, info};
use potboiler_common::{
    self, db, pg,
    types::{Log, CRDT},
};
use serde_derive::Deserialize;
use std::{
    collections::HashMap,
    env,
    ops::Deref,
    sync::{Arc, RwLock},
};

mod serde_types;
mod tables;

#[derive(Debug, Fail)]
enum KvError {
    #[fail(display = "WrongResultsCount")]
    WrongResultsCount { key: String, count: usize },
    #[fail(display = "UnsupportedCRDT")]
    UnsupportedCRDT { name: CRDT },
    #[fail(display = "UnsupportedORSETOp")]
    UnsupportedORSETOp { name: Operation },
    #[fail(display = "UnsupportedLWWOp")]
    UnsupportedLWWOp { name: Operation },
    #[fail(display = "NoTable")]
    NoTable { name: String },
    #[fail(display = "No such table '{}'", name)]
    NoSuchTable { name: String },
    #[fail(display = "No such key '{}'", name)]
    NoSuchKey { name: String },
    #[fail(display = "No key getter for {:?}", kind)]
    NoKeyGetter { kind: CRDT },
    #[fail(display = "DbError")]
    DbError {
        #[cause]
        cause: db::Error,
    },
}

impl ResponseError for KvError {
    fn error_response(&self) -> HttpResponse {
        let code = match *self {
            KvError::NoSuchKey { .. } | KvError::NoSuchTable { .. } => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        HttpResponse::build(code).body(format!("{}", self))
    }
}

lazy_static! {
    static ref SERVER_URL: String = env::var("SERVER_URL").expect("Needed SERVER_URL");
}

fn get_table_kind(state: &AppState, table: &str) -> Option<CRDT> {
    if table == tables::CONFIG_TABLE {
        Some(CRDT::LWW)
    } else {
        state.tables.get(table)
    }
}

#[derive(Deserialize)]
struct GetKeyPath {
    table: String,
    key: String,
}

fn get_key(path: Path<GetKeyPath>, state: State<AppState>) -> Result<HttpResponse, KvError> {
    let conn = state.pool.get().unwrap();
    let kind = match get_table_kind(state.deref(), &path.table) {
        Some(k) => k,
        None => {
            return Err(KvError::NoSuchTable {
                name: path.table.clone(),
            });
        }
    };
    match kind {
        CRDT::ORSET => {
            match conn.query(&format!(
                "SELECT key, item, metadata FROM {}_items where collection='{}'",
                path.table, path.key
            )) {
                Ok(rows) => {
                    let mut items = Vec::new();
                    for row in rows.iter() {
                        let key: String = row.get("key");
                        let item: String = row.get("item");
                        let metadata: serde_json::Value = row.get("metadata");
                        items.push(ORSetOp { item, key, metadata });
                    }
                    Ok(HttpResponse::Ok().json(items))
                }
                Err(db::Error::NoSuchTable) => Err(KvError::NoSuchTable {
                    name: path.table.clone(),
                }),

                Err(err) => {
                    error!(
                        "Error while getting key {} from table {}: {:?}",
                        path.key, path.table, err
                    );
                    Err(KvError::DbError { cause: err })
                }
            }
        }
        CRDT::LWW => match conn.query(&format!("SELECT value FROM {} where key='{}'", path.table, path.key)) {
            Ok(rows) => {
                if rows.is_empty() {
                    Err(KvError::NoSuchKey { name: path.key.clone() })
                } else {
                    let value: String = rows.get(0).get("value");
                    Ok(HttpResponse::Ok().body(value))
                }
            }
            Err(db::Error::NoSuchTable) => Err(KvError::NoSuchTable {
                name: path.table.clone(),
            }),
            Err(err) => {
                error!(
                    "Error while getting key {} from table {}: {:?}",
                    path.key, path.table, err
                );
                Err(KvError::DbError { cause: err })
            }
        },
        _ => Err(KvError::NoKeyGetter { kind }),
    }
}

#[derive(Deserialize, Debug)]
pub struct UpdateKeyPath {
    table: String,
    key: String,
}

fn update_key(json: Json<serde_json::Value>, path: Path<UpdateKeyPath>, state: State<AppState>) -> HttpResponse {
    let mut json_mut = json.clone();
    let map = json_mut.as_object_mut().unwrap();
    map.insert("table".to_string(), serde_json::to_value(path.table.clone()).unwrap());
    map.insert("key".to_string(), serde_json::to_value(path.key.clone()).unwrap());

    let change: Change = serde_json::from_value(json_mut).map_err(Error::from).unwrap();
    let send_change = change.clone();
    match change.op {
        Operation::Add | Operation::Remove => {
            serde_json::from_value::<ORSetOp>(change.change)
                .map_err(Error::from)
                .unwrap();
        }
        Operation::Create => {
            serde_json::from_value::<ORCreateOp>(change.change)
                .map_err(Error::from)
                .unwrap();
        }
        Operation::Set => {
            if change.table == tables::CONFIG_TABLE {
                serde_json::from_value::<LWWConfigOp>(change.change)
                    .map_err(Error::from)
                    .unwrap();
            }
        }
    }
    let res = state
        .client()
        .post(SERVER_URL.deref())
        .json(&send_change)
        .send()
        .expect("sender ok");
    assert_eq!(res.status(), reqwest::StatusCode::CREATED);
    HttpResponse::Ok().finish()
}

fn get_crdt(conn: &db::Connection, table: &str, key: &str) -> Result<Option<serde_json::Value>, Error> {
    let results = conn
        .query(&format!("select crdt from {} where key='{}'", table, key))
        .map_err(Error::from)?;
    if results.is_empty() {
        Ok(None)
    } else if results.len() == 1 {
        let row = results.get(0);
        let raw_crdt: serde_json::Value = row.get("crdt");
        Ok(Some(raw_crdt))
    } else {
        Err(KvError::WrongResultsCount {
            key: key.to_string(),
            count: results.len(),
        }
        .into())
    }
}

#[allow(clippy::map_entry)] // FIXME do maps better
fn new_event(state: State<AppState>, log: Json<Log>) -> Result<HttpResponse, Error> {
    info!("log: {:?}", log);
    let change: Change = serde_json::from_value(log.data.clone()).unwrap();
    info!("change: {:?}", change);
    let table_type = match state.tables.get(&change.table) {
        None => bail!(KvError::NoTable { name: change.table }),
        Some(val) => val,
    };
    match table_type {
        CRDT::LWW => match change.op {
            Operation::Set => {
                let conn = state.pool.get().unwrap();
                let raw_crdt = get_crdt(&conn, &change.table, &change.key).unwrap();
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
                            &serde_json::to_value(&lww).map_err(Error::from).unwrap()
                        ))
                        .map_err(Error::from)
                        .unwrap();
                        if change.table == tables::CONFIG_TABLE {
                            let config_op: LWWConfigOp = serde_json::from_value(change.change.clone())
                                .map_err(Error::from)
                                .unwrap();
                            tables::make_table(&conn, &change.key, config_op.crdt).unwrap();
                            state.tables.add(&change.key, config_op.crdt);
                        }
                    }
                    Some(raw_crdt) => {
                        let mut lww: LWW = serde_json::from_value(raw_crdt).map_err(Error::from).unwrap();
                        if lww.when < log.when {
                            lww.when = log.when;
                            lww.data = change.change.clone();
                            conn.execute(&format!(
                                "UPDATE {} set value='{}', crdt='{}' where key='{}'",
                                &change.table,
                                &change.change,
                                &change.key,
                                &serde_json::to_value(&lww).map_err(Error::from).unwrap()
                            ))
                            .unwrap();
                        } else {
                            info!("Earlier event, skipping");
                        }
                    }
                }
            }
            _ => {
                bail!(KvError::UnsupportedLWWOp { name: change.op });
            }
        },
        CRDT::ORSET => {
            let op: Option<ORSetOp> = match change.op {
                Operation::Add | Operation::Remove => {
                    Some(serde_json::from_value(change.change).map_err(Error::from).unwrap())
                }
                Operation::Create | Operation::Set => None,
            };
            let conn = state.pool.get().unwrap();
            let raw_crdt = get_crdt(&conn, &change.table, &change.key).unwrap();
            let (mut crdt, existing) = match raw_crdt {
                Some(val) => (serde_json::from_value(val).map_err(Error::from).unwrap(), true),
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
                            debug!("Updating '{}' in '{}/{}'", &unwrap_op.key, &change.table, &change.key);
                            let count = trans
                                .execute(&format!(
                                    "UPDATE {}_items set metadata='{}' where collection='{}' \
                                     and key='{}'",
                                    &change.table, &metadata, &change.key, &unwrap_op.key
                                ))
                                .unwrap();
                            if count != 1 {
                                error!(
                                    "Expected count 1 when updating '{}' in '{}', but got {}",
                                    &change.key, &change.table, count
                                );
                            }
                        } else {
                            debug!(
                                "Creating '{}' => '{}' in '{}/{}'",
                                &unwrap_op.key, &unwrap_op.item, &change.table, &change.key
                            );
                            trans
                                .execute(&format!(
                                    "INSERT INTO {}_items (collection, key, item, metadata) \
                                     VALUES ('{}', '{}', '{}', '{}')",
                                    &change.table, &change.key, &unwrap_op.key, &unwrap_op.item, &metadata
                                ))
                                .unwrap();
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
                        .unwrap();
                    crdt.adds.remove(&unwrap_op.key);
                    crdt.removes.insert(unwrap_op.key, unwrap_op.item);
                }
                Operation::Create => {
                    // Don't need to actually do anything to the item lists
                }
                _ => {
                    bail!(KvError::UnsupportedORSETOp { name: change.op });
                }
            }
            debug!("OR-Set for {} and {}: {:?}", &change.table, &change.key, &crdt);
            if existing {
                trans
                    .execute(&format!(
                        "UPDATE {} set crdt='{}' where key='{}'",
                        &change.table,
                        &serde_json::to_value(&crdt).unwrap(),
                        &change.key
                    ))
                    .unwrap();
            } else {
                trans
                    .execute(&format!(
                        "INSERT INTO {} (key, crdt) VALUES ('{}', '{}')",
                        &change.table,
                        &change.key,
                        &serde_json::to_value(&crdt).unwrap(),
                    ))
                    .unwrap();
            }
            let mut items: Vec<&str> = Vec::new();
            for key in crdt.adds.keys() {
                if crdt.removes.contains_key(key) {
                    continue;
                }
                items.push(&crdt.adds[key]);
            }
            debug!("Items: {:?}", items);
            //trans.commit()?;
        }
        _ => {
            bail!(KvError::UnsupportedCRDT { name: table_type });
        }
    }
    Ok(HttpResponse::NoContent().finish())
}

fn list_tables(state: State<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(state.tables.list())
}

#[derive(Deserialize, Debug)]
pub struct ListKeysReq {
    table: String,
}

fn list_keys(req: Path<ListKeysReq>, state: State<AppState>) -> HttpResponse {
    let conn = state.pool.get().unwrap();
    let mut key_names = vec![];
    for row in &conn.query(&format!("select key from {}", req.table)).unwrap() {
        let key: String = row.get("key");
        key_names.push(key);
    }
    HttpResponse::Ok().json(key_names)
}

pub fn db_setup() -> Result<db::Pool, Error> {
    let db_url: &str = &env::var("DATABASE_URL").expect("Needed DATABASE_URL");
    pg::get_pool(db_url).map_err(Error::from)
}

#[derive(Debug, Clone)]
pub struct AppState {
    pool: db::Pool,
    tables: tables::Tables,
    client: Arc<RwLock<reqwest::Client>>,
}

impl AppState {
    pub fn new(pool: db::Pool, client: reqwest::Client) -> Result<AppState, Error> {
        let conn = pool.get()?;
        let tables = tables::Tables::new(&conn)?;
        Ok(AppState {
            pool,
            tables,
            client: Arc::new(RwLock::new(client)),
        })
    }

    pub fn client(&self) -> reqwest::Client {
        self.client.read().unwrap().deref().clone()
    }
}

pub fn app_router(state: AppState) -> Result<App<AppState>, Error> {
    Ok(App::with_state(state)
        .resource("/kv", |r| r.method(Method::GET).with(list_tables))
        .resource("/kv/event", |r| r.method(Method::POST).with(new_event))
        .resource("/kv/{table}", |r| r.method(Method::GET).with(list_keys))
        .resource("/kv/{table}/{key}", |r| {
            r.method(Method::GET).with(get_key);
            r.method(Method::POST).with(update_key)
        }))
}

pub fn register(client: &reqwest::Client) -> Result<(), Error> {
    let mut map = serde_json::Map::new();
    let root: &str = &env::var("KV_ROOT").unwrap_or_else(|_| "http://localhost:8001/".to_string());
    map.insert(
        "url".to_string(),
        serde_json::Value::String(format!("{}kv/event", &root)),
    );
    let res = client
        .post(&format!("{}/register", SERVER_URL.deref()))
        .json(&map)
        .send()?;
    assert_eq!(res.status(), reqwest::StatusCode::CREATED);
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{app_router, db, AppState};
    use actix_web::{
        http::{Method, StatusCode},
        test, HttpMessage,
    };
    use serde_json::{json, Value};
    use std::str;

    fn setup_logging() {
        log4rs::init_file("log.yaml", Default::default()).unwrap();
    }

    fn test_get_route(server: &mut test::TestServer, path: &str, expected_body: &str, expected_status: StatusCode) {
        let request = server.client(Method::GET, path).finish().unwrap();
        let response = server.execute(request.send()).unwrap();
        assert_eq!(response.status(), expected_status);
        let bytes = server.execute(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        assert_eq!(body, expected_body);
    }

    fn test_post_route(
        server: &mut test::TestServer,
        path: &str,
        body: Value,
        expected_body: &str,
        expected_status: StatusCode,
    ) {
        let request = server.client(Method::POST, path).json(body).unwrap();
        let response = server.execute(request.send()).unwrap();
        assert_eq!(response.status(), expected_status);
        let bytes = server.execute(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        assert_eq!(body, expected_body);
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
        conn
    }

    fn setup_server(conn: db::TestConnection) -> test::TestServer {
        super::env::set_var("SERVER_URL", mockito::SERVER_URL);
        let pool = super::db::Pool::TestPool(conn);
        let app_state = AppState::new(pool, reqwest::Client::new()).unwrap();
        test::TestServer::with_factory(move || app_router(app_state.clone()).unwrap())
    }

    #[test]
    fn no_key_table() {
        let mut conn = setup_db(vec![]);
        conn.add_test_query("SELECT value FROM _config where key='test'", vec![]);
        let mut server = setup_server(conn);
        test_get_route(
            &mut server,
            "/kv/_config/test",
            "No such key 'test'",
            StatusCode::NOT_FOUND,
        );
    }

    #[test]
    fn key_table() {
        setup_logging();
        let conn = setup_db(vec![]);
        let mut server = setup_server(conn);
        let _mocks = vec![
            mockito::mock("POST", "/register").with_status(201).create(),
            mockito::mock("POST", "/").with_status(201).create(),
        ];
        test_post_route(
            &mut server,
            "/kv/_config/test",
            json!({
                "op":"set",
                "change":{"crdt": "LWW"}
            }),
            "",
            StatusCode::OK,
        );
    }
}
