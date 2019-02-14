#![recursion_limit = "128"]
#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    warnings,
    trivial_numeric_casts,
    unstable_features,
    unused,
    future_incompatible
)]

use error_chain::{
    // FIXME: Need https://github.com/rust-lang-nursery/error-chain/pull/253
    bail,
    error_chain,
    error_chain_processing,
    impl_error_chain_kind,
    impl_error_chain_processed,
    impl_extract_backtrace,
};
use iron::{self, prelude::*};
use log4rs;
use logger::{self, Logger};
use persistent::{self, Read as PRead, State};
use postgres;
use potboiler_common::{self, clock, db, pg};
use r2d2;
use router::{self, Router};
use schemamama;
use std::env;

mod logs;
mod nodes;
mod notifications;
mod schema;

error_chain! {
    errors {
        MigrationsOnNonPostgres(pool: db::Pool)
        IronError
    }
    links {
        NodeError(nodes::Error, nodes::ErrorKind);
        DbError(db::Error, db::ErrorKind);
    }
    foreign_links {
        PostgresError(r2d2::Error);
        SchemammaError(schemamama::Error<postgres::Error>);
        LogError(log4rs::Error);
    }
}

pub fn app_router(pool: db::Pool) -> Result<Chain> {
    let (logger_before, logger_after) = Logger::new(None);
    let mut router = Router::new();
    router.get("/log", logs::log_lasts, "last logs");
    router.post("/log", logs::new_log, "new log");
    router.post("/log/other", logs::other_log, "add from other");
    router.get("/log/first", logs::log_firsts, "get first logs");
    router.get("/log/:entry_id", logs::get_log, "get specific log");
    router.post("/log/register", notifications::log_register, "register log listener");
    router.post(
        "/log/deregister",
        notifications::log_deregister,
        "deregister log listener",
    );
    router.get("/nodes", nodes::node_list, "list other nodes");
    router.post("/nodes", nodes::node_add, "add new node");
    router.delete("/nodes", nodes::node_remove, "remove node");
    let mut chain = Chain::new(router);
    chain.link_before(logger_before);
    chain.link_after(logger_after);
    let conn = pool.get()?;
    chain.link_before(State::<notifications::Notifications>::one(
        notifications::init_notifiers(&conn),
    ));
    let clock_state = clock::init_clock();
    chain.link_before(State::<nodes::Nodes>::one(nodes::initial_nodes(
        pool.clone(),
        clock_state.clock_state.clone(),
    )?));
    chain.link_before(clock_state);
    chain.link(PRead::<db::PoolKey>::both(pool));
    Ok(chain)
}

pub fn db_setup() -> Result<db::Pool> {
    let db_url: &str = &env::var("DATABASE_URL").expect("Needed DATABASE_URL");
    let pool = pg::get_pool(db_url)?;
    if let db::Pool::Postgres(pg_pool) = pool {
        let conn = pg_pool.get()?;
        schema::up(&conn)?;
        Ok(db::Pool::Postgres(pg_pool))
    } else {
        bail!(ErrorKind::MigrationsOnNonPostgres(pool));
    }
}

#[cfg(test)]
mod test {
    use iron_test::{request, response::extract_body_to_string};

    use iron::{headers, status::Status, Headers};
    use serde_json;

    use regex::Regex;

    use super::{app_router, PRead};
    use potboiler_common::server_id;

    fn test_route(path: &str, expected: &str) {
        let mut conn = super::db::TestConnection::new();
        conn.add_test_query("select url from notifications", vec![]);
        conn.add_test_query("select url from nodes", vec![]);
        conn.add_test_query("select id, owner from log where next is null", vec![]);
        conn.add_test_query("select id, owner from log where prev is null", vec![]);
        let pool = super::db::Pool::TestPool(conn);
        let response = request::get(
            &format!("http://localhost:8000/{}", path),
            Headers::new(),
            &app_router(pool).unwrap(),
        )
        .unwrap();
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
        let mut conn = super::db::TestConnection::new();
        conn.add_test_query("select url from notifications", vec![]);
        conn.add_test_query("select url from nodes", vec![]);
        conn.add_test_query(
            concat!(
                "select id from log where next is null ",
                "and owner = 'feedface-dead-feed-face-deadfacedead' limit 1"
            ),
            vec![],
        );
        conn.add_test_execute(
            concat!(
                r"insert into log \(id, owner, data, prev, hlc_tstamp\) ",
                r"VALUES \('[a-z0-9-]+', 'feedface-dead-feed-face-deadfacedead', ",
                r"'\{\}', NULL, decode\('[0-9A-Z]+', 'hex'\)\)"
            ),
            1,
        );
        let pool = super::db::Pool::TestPool(conn);
        let mut router = app_router(pool).unwrap();
        router.link_before(PRead::<server_id::ServerId>::one(server_id::test()));
        let response = request::post("http://localhost:8000/log", Headers::new(), "{}", &router).unwrap();
        assert_eq!(response.status.unwrap(), Status::Created);
        let uuid = {
            let re = Regex::new(r"http://localhost:8000/log/([a-z0-9-]+)").unwrap();
            let url = String::from(response.headers.get::<headers::Location>().unwrap().as_str());
            assert!(url.starts_with("http://localhost:8000/log/"));
            String::from(re.captures(&url).unwrap().get(1).unwrap().as_str())
        };
        let result = extract_body_to_string(response);
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(v.is_object());
        assert_eq!(v["id"].as_str().unwrap(), uuid);
    }
}
