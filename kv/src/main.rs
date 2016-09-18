#[macro_use]
extern crate log;
extern crate log4rs;

extern crate iron;
extern crate logger;
extern crate router;
use iron::prelude::*;
use iron::status;
use logger::Logger;
use router::Router;

extern crate persistent;
use persistent::Read as PRead;

#[macro_use]
extern crate potboiler_common;
use potboiler_common::db;
use potboiler_common::server_id;

use std::env;

fn get_key(req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "get_key")))
}

fn update_key(req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "update_key")))
}

fn main() {
    log4rs::init_file("log.yaml", Default::default()).unwrap();
    let db_url: &str = &env::var("DATABASE_URL").expect("Needed DATABASE_URL");
    let pool = db::get_pool(db_url);
    // let conn = pool.get().unwrap();
    //    schema::up(&conn).unwrap();
    let (logger_before, logger_after) = Logger::new(None);
    let mut router = Router::new();
    router.get("/kv/:table/:key", get_key);
    router.post("/kv/:table/:key", update_key);
    let mut chain = Chain::new(router);
    chain.link_before(logger_before);
    chain.link_after(logger_after);
    chain.link_before(PRead::<server_id::ServerId>::one(server_id::setup()));
    chain.link(PRead::<db::PostgresDB>::both(pool));
    info!("Potboiler-kv booted");
    Iron::new(chain).http("localhost:8001").unwrap();
}
