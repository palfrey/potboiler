#[macro_use]
extern crate schemamama;
extern crate schemamama_postgres;
extern crate postgres;
mod schema;

#[macro_use]
extern crate log;
extern crate log4rs;

extern crate iron;
extern crate router;
extern crate logger;

use iron::prelude::*;
use iron::status;
use router::Router;
use logger::Logger;

use std::env;

extern crate uuid;
use uuid::Uuid;
extern crate json;
use json::JsonValue;

extern crate r2d2;
extern crate r2d2_postgres;
extern crate persistent;
use persistent::Read as PRead;

#[macro_use]
mod db;

struct Log {
    id: Uuid,
    owner: Uuid,
    next: Option<Uuid>,
    prev: Option<Uuid>,
    data: JsonValue
}

fn log_status(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Hello World!")))
}

fn new_log(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "New log")))
}

fn get_log(req: &mut Request) -> IronResult<Response> {
    let ref query = req.extensions
        .get::<Router>()
        .unwrap()
        .find("entry_id")
        .unwrap_or("/");
    let conn = get_pg_connection!(&req);
    let stmt = conn.prepare("SELECT data from log").unwrap();
    let results = stmt.query(&[]).unwrap();
    if results.is_empty() {
        Ok(Response::with((status::NotFound, format!("No log {}", query))))
    }
    else {
        Ok(Response::with((status::Ok, format!("Get log {}", query))))
    }
}

fn main() {
    log4rs::init_file("log.yaml", Default::default()).unwrap();
    let db_url: &str = &env::var("DATABASE_URL").expect("Needed DATABASE_URL");
    let pool = db::get_pool(db_url);
    let conn = pool.get().unwrap();
    schema::up(&conn).unwrap();
    let (logger_before, logger_after) = Logger::new(None);
    let mut router = Router::new();
    router.get("/log", log_status);
    router.post("/log", new_log);
    router.get("/log/:entry_id", get_log);
    let mut chain = Chain::new(router);
    chain.link_before(logger_before);
    chain.link_after(logger_after);
    chain.link(PRead::<db::PostgresDB>::both(pool));
    Iron::new(chain).http("localhost:8000").unwrap();
}
