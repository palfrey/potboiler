#[macro_use]
extern crate schemamama;
extern crate schemamama_postgres;
extern crate postgres;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate iron;
extern crate router;
extern crate logger;
extern crate hyper;
extern crate url;
extern crate uuid;
extern crate serde;
extern crate serde_json;
extern crate hybrid_clocks;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate persistent;
#[macro_use]
extern crate potboiler_common;
extern crate urlencoded;
extern crate plugin;
extern crate resolve;

use iron::prelude::*;
use logger::Logger;
use persistent::Read as PRead;
use persistent::State;
use potboiler_common::db;
use potboiler_common::server_id;
use router::Router;
use std::env;
mod notifications;
mod clock;
mod nodes;
mod logs;
mod schema;

fn main() {
    log4rs::init_file("log.yaml", Default::default()).unwrap();
    let db_url: &str = &env::var("DATABASE_URL").expect("Needed DATABASE_URL");
    let pool = db::get_pool(db_url);
    let conn = pool.get().unwrap();
    schema::up(&conn).unwrap();
    let (logger_before, logger_after) = Logger::new(None);
    let mut router = Router::new();
    router.get("/log", logs::log_lasts);
    router.post("/log", logs::new_log);
    router.post("/log/other", logs::other_log);
    router.get("/log/first", logs::log_firsts);
    router.get("/log/:entry_id", logs::get_log);
    router.post("/log/register", notifications::log_register);
    router.post("/log/deregister", notifications::log_deregister);
    router.get("/nodes", nodes::node_list);
    router.post("/nodes", nodes::node_add);
    router.delete("/nodes", nodes::node_remove);
    let mut chain = Chain::new(router);
    chain.link_before(logger_before);
    chain.link_after(logger_after);
    chain.link_before(State::<notifications::Notifications>::one(notifications::init_notifiers(&conn)));
    let clock_state = clock::init_clock();
    chain.link_before(State::<nodes::Nodes>::one(nodes::initial_nodes(pool.clone(),
                                                                      clock_state.clock_state.clone())));
    chain.link_before(clock_state);
    chain.link_before(PRead::<server_id::ServerId>::one(server_id::setup()));
    chain.link(PRead::<db::PostgresDB>::both(pool));
    info!("Potboiler booted");
    Iron::new(chain).http("0.0.0.0:8000").unwrap();
}
