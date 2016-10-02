use iron::Request;
use iron::typemap::Key;
use persistent::State;
use r2d2::PooledConnection;
use r2d2_postgres::PostgresConnectionManager;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
pub type PostgresConnection = PooledConnection<PostgresConnectionManager>;
use CRDT;
use serde_json;
use serde_json::Value;

#[derive(Copy, Clone)]
pub struct Tables;

impl Key for Tables {
    type Value = HashMap<String, CRDT>;
}

pub fn get_tables(req: &Request) -> HashMap<String, CRDT> {
    req.extensions.get::<State<Tables>>().unwrap().write().unwrap().deref().clone()
}

pub fn init_tables(conn: &PostgresConnection) -> HashMap<String, CRDT> {
    let mut tables: HashMap<String, CRDT> = HashMap::new();
    tables.insert("_config".to_string(), CRDT::LWW);
    let stmt = conn.prepare("select key, value from _config").expect("prepare failure");
    for row in &stmt.query(&[]).expect("last select works") {
        let key: String = row.get("key");
        let value: Value = row.get("value");
        tables.insert(key.to_string(),
                      serde_json::from_value(value.find("crdt").unwrap().clone()).unwrap());
    }
    tables
}

pub fn add_table(req: &mut Request, table_name: String, crdt_type: CRDT) {
    req.extensions
        .get_mut::<State<Tables>>()
        .unwrap()
        .write()
        .unwrap()
        .deref_mut()
        .insert(table_name, crdt_type);
}
