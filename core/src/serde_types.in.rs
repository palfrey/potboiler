use hybrid_clocks::{Timestamp, WallT};
use uuid::Uuid;
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct Log {
    pub id: Uuid,
    pub owner: Uuid,
    pub prev: Option<Uuid>,
    pub next: Option<Uuid>,
    pub when: Timestamp<WallT>,
    pub data: serde_json::Value
}
