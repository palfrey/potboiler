extern crate serde;
use std::collections::HashMap;

enum_str!(Operation {
    Set("set"),
    Add("add"),
    Remove("remove"),
    Create("create"),
});

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Change {
    table: String,
    key: String,
    op: Operation,
    change: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct LWWConfigOp {
    crdt: CRDT
}

#[derive(Serialize, Deserialize, Debug)]
struct ORCreateOp {
}

#[derive(Serialize, Deserialize, Debug)]
struct ORSetOp {
    item: String,
    key: String
}

#[derive(Serialize, Deserialize, Debug)]
struct ORSet {
    adds: HashMap<String, String>,
    removes: HashMap<String, String>
}