use uuid::Uuid;
use serde_json;

#[derive(Debug, Queryable)]
pub struct Log {
    pub id: Uuid,
    pub owner: Uuid,
    pub prev: Option<Uuid>,
    pub next: Option<Uuid>,
    pub data: serde_json::Value,
    pub hlc_tstamp: Vec<u8>,
}