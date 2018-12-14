use iron;
use iron::status::Status;
use iron::Headers;
use iron_test;
use iron_test::request;
use iron_test::response::extract_body_to_string;
use persistent;
use persistent::Read as PRead;
use potboiler;
use potboiler_common;
use potboiler_common::server_id;

#[test]
#[ignore] // because needs db
fn test_pg_router() {
    let pool = potboiler::db_setup().unwrap();
    pool.get().unwrap().execute("delete from log").unwrap();
    let mut router = potboiler::app_router(pool).unwrap();
    router.link_before(PRead::<server_id::ServerId>::one(server_id::test()));
    let mut response = request::get("http://localhost:8000/log", Headers::new(), &router).unwrap();
    assert_eq!(response.status.unwrap(), Status::Ok);
    let result = extract_body_to_string(response);
    assert_eq!(result, "{}");

    response = request::post("http://localhost:8000/log", Headers::new(), "{}", &router).unwrap();
    assert_eq!(response.status.unwrap(), Status::Created);
}
