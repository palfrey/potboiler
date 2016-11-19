use iron::Request;
use iron::typemap::Key;
use persistent::State;
use potboiler_common::types::CRDT;
use r2d2::PooledConnection;
use r2d2_postgres::PostgresConnectionManager;
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

pub type PostgresConnection = PooledConnection<PostgresConnectionManager>;

#[derive(Copy, Clone)]
pub struct Tables;

impl Key for Tables {
    type Value = HashMap<String, CRDT>;
}

pub fn get_tables(req: &Request) -> HashMap<String, CRDT> {
    req.extensions.get::<State<Tables>>().unwrap().write().unwrap().deref().clone()
}

pub static CONFIG_TABLE: &'static str = "_config";

pub fn init_tables(conn: &PostgresConnection) -> HashMap<String, CRDT> {
    let mut tables: HashMap<String, CRDT> = HashMap::new();
    tables.insert(CONFIG_TABLE.to_string(), CRDT::LWW);
    let stmt = conn.prepare(&format!("select key, value from {}", CONFIG_TABLE)).expect("prepare failure");
    for row in &stmt.query(&[]).expect("last select works") {
        let key: String = row.get("key");
        let value: Value = row.get("value");
        tables.insert(key.to_string(),
                      serde_json::from_value(value.find("crdt").unwrap().clone()).unwrap());
    }
    tables
}

pub fn add_table(req: &mut Request, table_name: &String, crdt_type: CRDT) {
    req.extensions
        .get_mut::<State<Tables>>()
        .unwrap()
        .write()
        .unwrap()
        .deref_mut()
        .insert(table_name.clone(), crdt_type);
}
