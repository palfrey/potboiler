use iron::{typemap::Key, Request};
use persistent::State;
use potboiler_common::{db, types::CRDT};
use serde_json::{self, Value};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

#[derive(Copy, Clone)]
pub struct Tables;

impl Key for Tables {
    type Value = HashMap<String, CRDT>;
}

pub fn get_tables(req: &Request) -> HashMap<String, CRDT> {
    req.extensions
        .get::<State<Tables>>()
        .unwrap()
        .write()
        .unwrap()
        .deref()
        .clone()
}

pub static CONFIG_TABLE: &'static str = "_config";

pub fn init_tables(conn: &db::Connection) -> HashMap<String, CRDT> {
    let mut tables: HashMap<String, CRDT> = HashMap::new();
    tables.insert(CONFIG_TABLE.to_string(), CRDT::LWW);
    for row in &conn
        .query(&format!("select key, value from {}", CONFIG_TABLE))
        .expect("last select works")
    {
        let key: String = row.get("key");
        let value: Value = row.get("value");
        tables.insert(
            key.to_string(),
            serde_json::from_value(value.get("crdt").unwrap().clone()).unwrap(),
        );
    }
    tables
}

pub fn add_table(req: &mut Request, table_name: &str, crdt_type: CRDT) {
    req.extensions
        .get_mut::<State<Tables>>()
        .unwrap()
        .write()
        .unwrap()
        .deref_mut()
        .insert(table_name.to_string(), crdt_type);
}
