use potboiler_common::enum_str;
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueCreate {
    pub name: String,
    pub timeout_ms: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueConfig {
    pub timeout_ms: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueAdd {
    pub queue_name: String,
    pub task_name: String,
    pub info: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueProgress {
    pub queue_name: String,
    pub id: Uuid,
    pub worker_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum QueueOperation {
    Create(QueueCreate),
    Delete(String),
    Add(QueueAdd),
    Progress(QueueProgress),
    Done(QueueProgress),
}

enum_str!(QueueState {
    Pending("pending"),
    Working("working"),
    Done("done"),
});

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueListItem {
    pub task_name: String,
    pub state: QueueState,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueItem {
    pub task_name: String,
    pub state: QueueState,
    pub info: serde_json::Value,
    pub worker: Option<Uuid>,
}
