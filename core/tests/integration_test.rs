use iron::{self, status::Status, Headers, Chain};
use iron_test::{self, request, response::extract_body_to_string};
use persistent::{self, Read as PRead};
use potboiler;
use potboiler_common::{self, server_id};
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::cell::RefCell;
use antidote;

thread_local!(static LOCKS: RefCell<HashMap<String, antidote::Mutex<()>>> = RefCell::new(HashMap::new()));

fn serial(name: &str, function: fn()) {
    LOCKS.with(|ll| {
        let mut local_lock = ll.borrow_mut();
        if !local_lock.contains_key(name) {
            local_lock.insert(name.to_string(), antidote::Mutex::new(()));
        }
        let _guard = local_lock[name].lock();
        function();
    });
}

fn test_setup() -> Chain {
    let pool = potboiler::db_setup().unwrap();
    pool.get().unwrap().execute("delete from log").unwrap();
    let mut router = potboiler::app_router(pool).unwrap();
    router.link_before(PRead::<server_id::ServerId>::one(server_id::test()));
    return router;
}

#[test]
fn test_empty_log() {
    serial("db", || {
        let router = test_setup();
        let response = request::get("http://localhost:8000/log", Headers::new(), &router).unwrap();
        assert_eq!(response.status.unwrap(), Status::Ok);
        let result = extract_body_to_string(response);
        assert_eq!(result, "{}");
    });
}

#[test]
fn test_create() {
    serial("db", || {
        let router = test_setup();
        let mut response = request::post("http://localhost:8000/log", Headers::new(), "{}", &router).unwrap();
        assert_eq!(response.status.unwrap(), Status::Created);
        response = request::get("http://localhost:8000/log", Headers::new(), &router).unwrap();
        assert_eq!(response.status.unwrap(), Status::Ok);
        let result = extract_body_to_string(response);
        assert_eq!(result, "{\"feedface-dead-feed-face-deadfacedead\":\"28f4b0c4-4149-49d5-a0e6-52a1fc8f2e56\"}");
    });
}
