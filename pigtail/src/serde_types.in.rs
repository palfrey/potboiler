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
pub struct QueueAdd {
    pub queue_name: String,
    pub task_name: String,
    pub data: serde_json::Value
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueProgress {
    pub queue_name: String,
    pub id: Uuid,
    pub worker_id: Uuid
}

#[derive(Serialize, Deserialize, Debug)]
pub enum QueueOperation {
    Create(QueueCreate),
    Delete(String),
    Add(QueueAdd),
    Progress(QueueProgress),
    Done(QueueProgress)
}

enum_str!(QueueState {
    Pending("pending"),
    Working("working"),
    Done("done"),
});

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueListItem {
    pub task_name: String,
    pub state: QueueState
}
