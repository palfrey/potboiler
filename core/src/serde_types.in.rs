use hybrid_clocks::{Timestamp, WallT};

#[derive(Serialize, Deserialize, Debug)]
struct Log {
    id: Uuid,
    owner: Uuid,
    prev: Option<Uuid>,
    next: Option<Uuid>,
    when: Timestamp<WallT>,
    data: serde_json::Value
}
