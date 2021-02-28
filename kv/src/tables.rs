use anyhow::Result;
use potboiler_common::{db, types::CRDT};
use serde_json::{self, Value};
use std::{
    collections::HashMap,
    ops::DerefMut,
    sync::{Arc, RwLock},
};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Tables {
    tables: Arc<RwLock<HashMap<String, CRDT>>>,
}

pub static CONFIG_TABLE: &str = "_config";

#[derive(Debug, Error)]
pub enum TableError {
    #[error("ConfigTableCreation")]
    ConfigTableCreation {
        #[source]
        cause: anyhow::Error,
    },
}

pub fn make_table(conn: &db::Connection, table_name: &str, kind: CRDT) -> Result<()> {
    match kind {
        CRDT::LWW => {
            conn.execute(&format!(
                "CREATE TABLE IF NOT EXISTS {} (key VARCHAR(1024) PRIMARY KEY, value \
                 JSONB NOT NULL, crdt JSONB NOT NULL)",
                &table_name
            ))?;
        }
        CRDT::ORSET => {
            conn.execute(&format!(
                "CREATE TABLE IF NOT EXISTS {} (key VARCHAR(1024) PRIMARY KEY, crdt \
                 JSONB NOT NULL)",
                &table_name
            ))?;
            conn.execute(&format!(
                "CREATE TABLE IF NOT EXISTS {}_items (\
                                   collection VARCHAR(1024) NOT NULL,
                                   key VARCHAR(1024) NOT NULL, \
                                   item VARCHAR(1024) NOT NULL, \
                                   metadata JSONB NOT NULL, \
                                   PRIMARY KEY(collection, key, item))",
                &table_name
            ))?;
        }
        CRDT::GSET => {
            anyhow::anyhow!("No G-Set make table yet");
        }
    }
    Ok(())
}

impl Tables {
    pub fn new(conn: &db::Connection) -> Result<Tables, TableError> {
        match make_table(&conn, CONFIG_TABLE, CRDT::LWW) {
            Ok(_) => {}
            Err(err) => {
                return Err(TableError::ConfigTableCreation { cause: err });
            }
        }
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
        Ok(Tables {
            tables: Arc::new(RwLock::new(tables)),
        })
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
