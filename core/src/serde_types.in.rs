#[derive(Serialize, Deserialize, Debug)]
struct Log {
    id: Uuid,
    owner: Uuid,
    prev: Option<Uuid>,
    next: Option<Uuid>,
    data: serde_json::Value
}
