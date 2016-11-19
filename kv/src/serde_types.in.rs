extern crate serde;
use hybrid_clocks::{Timestamp, WallT};

enum_str!(Operation {
    Set("set"),
    Add("add"),
    Remove("remove"),
});

#[derive(Serialize, Deserialize, Debug)]
struct Change {
    table: String,
    key: String,
    op: Operation,
    change: serde_json::Value,
}
