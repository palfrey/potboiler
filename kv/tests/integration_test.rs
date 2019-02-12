use iron::{self, status::Status, Chain, Headers};
use iron_test::{self, request, response::extract_body_to_string};
use persistent::{self, Read as PRead};
use kv;
use potboiler_common::{self, server_id};
use pretty_assertions::assert_eq;
use serial_test_derive::serial;

fn test_setup() -> Chain {
    log4rs::init_file("log.yaml", Default::default()).unwrap();
    let pool = kv::db_setup().unwrap();
    let conn = pool.get().unwrap();
    conn.execute("delete from log").unwrap();
    conn.execute("delete from nodes").unwrap();
    conn.execute("delete from _config").unwrap();
    let mut router = kv::app_router(pool).unwrap();
    router.link_before(PRead::<server_id::ServerId>::one(server_id::test()));
    return router;
}

#[test]
#[serial]
fn test_empty_table() {
    let router = test_setup();
    let response = request::get("http://localhost:8000/kv/_config/test", Headers::new(), &router).unwrap();
    assert_eq!(response.status.unwrap(), Status::NotFound);
}