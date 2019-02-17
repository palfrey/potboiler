use potboiler_common::{db, types::CRDT};
use serde_json::{self, Value};
use std::{
    collections::HashMap,
    ops::DerefMut,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone)]
pub struct Tables {
    tables: Arc<RwLock<HashMap<String, CRDT>>>,
}

pub static CONFIG_TABLE: &'static str = "_config";

impl Tables {
    pub fn new(conn: &db::Connection) -> Tables {
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
        Tables {
            tables: Arc::new(RwLock::new(tables)),
        }
    }

    pub fn list(&self) -> Vec<String> {
        let mut table_names = vec![];
        for t in self.tables.read().unwrap().keys() {
            table_names.push(t.clone());
        }
        table_names
    }

    pub fn get(&self, name: &str) -> Option<CRDT> {
        self.tables.read().unwrap().get(name).cloned()
    }

    pub fn add(&self, table_name: &str, crdt_type: CRDT) {
        self.tables
            .write()
            .unwrap()
            .deref_mut()
            .insert(table_name.to_string(), crdt_type);
    }
}
