#![recursion_limit="128"]
#![deny(missing_debug_implementations, missing_copy_implementations,
        warnings,
        trivial_numeric_casts,
        unstable_features,
        unused, future_incompatible)]

// #[macro_use]
// extern crate schemamama;
// extern crate schemamama_postgres;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate iron;
extern crate router;
extern crate logger;
extern crate hyper;
extern crate url;
extern crate uuid;
extern crate serde_json;
extern crate hybrid_clocks;
extern crate r2d2;
extern crate persistent;
#[macro_use]
extern crate potboiler_common;
extern crate urlencoded;
extern crate plugin;
extern crate resolve;
#[macro_use]
extern crate diesel;
extern crate r2d2_diesel;
#[macro_use]
extern crate error_chain;

use iron::prelude::*;
use logger::Logger;
use persistent::Read as PRead;
use persistent::State;
use potboiler_common::{clock, db, pg, server_id};
use router::Router;
use std::env;

mod notifications;
mod nodes;
mod logs;
mod schema;
mod models;

error_chain! {
    errors {
        MigrationsOnNonPostgres(pool: String)
    }
    links {
        NodeError(nodes::Error, nodes::ErrorKind);
        DbError(db::Error, db::ErrorKind);
    }
    foreign_links {
        R2D2Error(r2d2::Error);
        LogError(log4rs::Error);
        HyperError(hyper::Error);
    }
}

fn app_router(pool: db::Pool) -> Result<Chain> {
    let (logger_before, logger_after) = Logger::new(None);
    let mut router = Router::new();
    router.get("/log", logs::log_lasts, "last logs");
    router.post("/log", logs::new_log::, "new log");
    router.post("/log/other", logs::other_log, "add from other");
    router.get("/log/first", logs::log_firsts, "get first logs");
    router.get("/log/:entry_id", logs::get_log, "get specific log");
    router.post("/log/register",
                notifications::log_register,
                "register log listener");
    router.post("/log/deregister",
                notifications::log_deregister,
                "deregister log listener");
    router.get("/nodes", nodes::node_list, "list other nodes");
    router.post("/nodes", nodes::node_add, "add new node");
    router.delete("/nodes", nodes::node_remove, "remove node");
    let mut chain = Chain::new(router);
    chain.link_before(logger_before);
    chain.link_after(logger_after);
    chain.link_before(PRead::<server_id::ServerId>::one(server_id::setup()));
    let conn = pool.get()?;
    chain.link_before(State::<notifications::Notifications>::one(notifications::init_notifiers(&conn)));
    let clock_state = clock::init_clock();
    chain.link_before(State::<nodes::Nodes>::one(nodes::initial_nodes(pool.clone(),
                                                                      clock_state.clock_state.clone())?));
    chain.link_before(clock_state);
    chain.link(PRead::<db::PoolKey>::both(pool));
    return Ok(chain);
}

fn db_setup() -> Result<db::Pool<diesel::PgConnection>> {
    let db_url: &str = &env::var("DATABASE_URL").expect("Needed DATABASE_URL");
    let pool = pg::get_pool(db_url)?;
    if let db::Pool::Postgres(pg_pool) = pool {
        let conn = pg_pool.get()?;
        //schema::up(&conn)?;
    }
    else {
        bail!(ErrorKind::MigrationsOnNonPostgres(pool));
    }
    return Ok(pool);
}

quick_main!(|| -> Result<()> {
    log4rs::init_file("log.yaml", Default::default())?;
    let pool = db_setup()?;
    let chain = app_router(pool)?;
    info!("Potboiler booted");
    Iron::new(chain).http("0.0.0.0:8000")?;
    Ok(())
});

#[cfg(test)]
mod test {
    extern crate iron_test;
    use iron::Headers;
    use iron_test::response::extract_body_to_string;
    use iron_test::request;
    use iron::status::Status;

    use super::{app_router};

    #[test]
    fn test_router() {
        let response = request::get("http://localhost:8000/log",
                                    Headers::new(),
                                    &app_router()).unwrap();
        assert_eq!(response.status.unwrap(), Status::Ok);
        let result = extract_body_to_string(response);

        assert_eq!(result, "[]");
    }
}
