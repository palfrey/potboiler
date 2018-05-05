#![recursion_limit="128"]
#![deny(missing_debug_implementations, missing_copy_implementations,
        warnings,
        trivial_numeric_casts,
        unstable_features,
        unused, future_incompatible)]

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
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate hybrid_clocks;
extern crate r2d2;
extern crate persistent;
#[macro_use]
extern crate potboiler_common;
extern crate urlencoded;
extern crate plugin;
extern crate resolve;
extern crate deuterium;
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

error_chain! {
    errors {
        MigrationsOnNonPostgres(pool: db::Pool)
    }
    links {
        NodeError(nodes::Error, nodes::ErrorKind);
        DbError(db::Error, db::ErrorKind);
    }
    foreign_links {
        PostgresError(r2d2::Error);
        SchemammaError(schemamama::Error<postgres::Error>);
        LogError(log4rs::Error);
        HyperError(hyper::Error);
    }
}

fn app_router(pool: db::Pool) -> Result<Chain> {
    let (logger_before, logger_after) = Logger::new(None);
    let mut router = Router::new();
    router.get("/log", logs::log_lasts, "last logs");
    router.post("/log", logs::new_log, "new log");
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

fn db_setup() -> Result<db::Pool> {
    let db_url: &str = &env::var("DATABASE_URL").expect("Needed DATABASE_URL");
    let pool = pg::get_pool(db_url)?;
    if let db::Pool::Postgres(pg_pool) = pool {
        let conn = pg_pool.get()?;
        schema::up(&conn)?;
        return Ok(db::Pool::Postgres(pg_pool));
    }
    else {
        bail!(ErrorKind::MigrationsOnNonPostgres(pool));
    }
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
    use self::iron_test::response::extract_body_to_string;
    use self::iron_test::request;

    use iron::Headers;
    use iron::headers;
    use iron::status::Status;
    use serde_json;

    extern crate regex;
    use self::regex::Regex;

    use super::{app_router};

    fn test_route(path: &str, expected: &str) {
        let pool = super::db::Pool::TestPool(super::db::TestConnection);
        let response = request::get(&format!("http://localhost:8000/{}", path),
                                    Headers::new(),
                                    &app_router(pool).unwrap()).unwrap();
        assert_eq!(response.status.unwrap(), Status::Ok);
        let result = extract_body_to_string(response);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_log() {
        test_route("log", "{}");
    }

    #[test]
    fn test_nodes() {
        test_route("nodes", "[]");
    }

    #[test]
    fn test_log_first() {
        test_route("log/first", "{}");
    }

    #[test]
    fn test_new_log() {
        let pool = super::db::Pool::TestPool(super::db::TestConnection);
        let response = request::post("http://localhost:8000/log",
                                    Headers::new(),
                                    "{}",
                                    &app_router(pool).unwrap()).unwrap();
        assert_eq!(response.status.unwrap(), Status::Created);
        let uuid = {
            let re = Regex::new(r"http://localhost:8000/log/([a-z0-9-]+)").unwrap();
            let url = String::from(response.headers.get::<headers::Location>().unwrap().as_str());
            assert!(url.starts_with("http://localhost:8000/log/"));
            String::from(re.captures(&url).unwrap().at(1).unwrap())
        };
        let result = extract_body_to_string(response);
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(v.is_object());
        assert_eq!(v["id"].as_str().unwrap(), uuid);
    }

    #[test]
    fn test_pg_router() {
        let pool = super::db_setup().unwrap();
        let response = request::get("http://localhost:8000/log",
                                    Headers::new(),
                                    &app_router(pool).unwrap()).unwrap();
        assert_eq!(response.status.unwrap(), Status::Ok);
        let result = extract_body_to_string(response);

        assert_eq!(result, "{}");
    }
}
