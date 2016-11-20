use uuid::Uuid;
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueCreate {
    pub name: String,
    pub timeout_ms: i32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueConfig {
    pub timeout_ms: i32
}

#[derive(Serialize, Deserialize, Debug)]
pub enum QueueOperation {
    Create(QueueCreate),
    Delete { name: String },
    Add { data: serde_json::Value },
    Progress { queue_id: Uuid, worker_id: Uuid },
    Done { queue_id: Uuid, worker_id: Uuid }
}
