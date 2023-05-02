use hybrid_clocks::{Timestamp, WallMST};
use potboiler_common::{enum_str, types::CRDT};
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

enum_str!(Operation {
    Set("set"),
    Add("add"),
    Remove("remove"),
    Create("create"),
});

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Change {
    pub table: String,
    pub key: String,
    pub op: Operation,
    pub change: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LWWConfigOp {
    pub crdt: CRDT,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LWW {
    pub when: Timestamp<WallMST>,
    pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ORCreateOp {}

#[derive(Serialize, Deserialize, Debug)]
pub struct ORSetOp {
    pub item: String,
    pub key: String,
    pub metadata: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ORSet {
    pub adds: HashMap<String, String>,
    pub removes: HashMap<String, String>,
}
