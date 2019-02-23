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

use actix_web::{http::Method, App};
use failure::{bail, Error, Fail};
use potboiler_common::{self, clock, db, pg};
use std::env;

mod logs;
mod nodes;
mod notifications;
mod schema;

#[derive(Debug, Fail)]
enum CoreError {
    #[fail(display = "migration on non-postgres: {:?}", pool)]
    MigrationsOnNonPostgres { pool: db::Pool },
}

#[derive(Debug, Clone)]
pub struct AppState {
    server_id: uuid::Uuid,
    clock: clock::SyncClock,
    pool: db::Pool,
    nodes: nodes::NodeList,
    notifications: notifications::Notifications,
}

impl AppState {
    pub fn new(pool: db::Pool, server: uuid::Uuid) -> Result<AppState, Error> {
        let clock = clock::SyncClock::new();
        Ok(AppState {
            server_id: server,
            clock: clock.clone(),
            pool: pool.clone(),
            nodes: nodes::initial_nodes(pool.clone(), clock.clone())?,
            notifications: notifications::Notifications::new(&pool.get().unwrap()),
        })
    }
}

pub fn app_router(state: AppState) -> Result<App<AppState>, Error> {
    Ok(App::with_state(state)
        .resource("/log", |r| {
            r.method(Method::GET).with(logs::log_lasts);
            r.method(Method::POST).with(logs::new_log);
        })
        .resource("/log/other", |r| r.method(Method::POST).with(logs::other_log))
        .resource("/log/first", |r| r.method(Method::GET).with(logs::log_firsts))
        .resource("/log/register", |r| {
            r.method(Method::POST).with(notifications::log_register)
        })
        .resource("/log/deregister", |r| {
            r.method(Method::POST).with(notifications::log_deregister)
        })
        .resource("/log/{entry_id}", |r| r.method(Method::GET).with(logs::get_log))
        .resource("/nodes", |r| {
            r.method(Method::GET).with(nodes::node_list);
            r.method(Method::POST).with(nodes::node_add);
            r.method(Method::DELETE).with(nodes::node_remove);
        }))
}

pub fn db_setup() -> Result<db::Pool, Error> {
    let db_url: &str = &env::var("DATABASE_URL").expect("Needed DATABASE_URL");
    let pool = pg::get_pool(db_url)?;
    if let db::Pool::Postgres(pg_pool) = pool {
        let conn = pg_pool.get()?;
        schema::up(&conn)?;
        Ok(db::Pool::Postgres(pg_pool))
    } else {
        bail!(CoreError::MigrationsOnNonPostgres { pool });
    }
}

#[cfg(test)]
mod test {
    use super::app_router;
    use actix_web::{http::header, http::Method, http::StatusCode, test, HttpMessage};
    use potboiler_common::server_id;
    use regex::Regex;
    use serde_json;
    use std::str;

    fn test_route(path: &str, expected: &str) {
        let _ = env_logger::try_init();
        let mut conn = super::db::TestConnection::new();
        conn.add_test_query("select url from notifications", vec![]);
        conn.add_test_query("select url from nodes", vec![]);
        conn.add_test_query("select id, owner from log where next is null", vec![]);
        conn.add_test_query("select id, owner from log where prev is null", vec![]);
        let pool = super::db::Pool::TestPool(conn);
        let app_state = super::AppState::new(pool, server_id::test()).unwrap();
        let mut server = test::TestServer::with_factory(move || app_router(app_state.clone()).unwrap());
        let request = server.client(Method::GET, path).finish().unwrap();
        let response = server.execute(request.send()).unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = server.execute(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        assert_eq!(body, expected);
    }

    #[test]
    fn test_log() {
        test_route("/log", "{}");
    }

    #[test]
    fn test_nodes() {
        test_route("/nodes", "[]");
    }

    #[test]
    fn test_log_first() {
        test_route("/log/first", "{}");
    }

    #[test]
    fn test_new_log() {
        let _ = env_logger::try_init();
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
        conn.add_test_query(
            concat!(r"select id from dependency where depends_on = '[a-z0-9-]+'"),
            vec![],
        );
        let pool = super::db::Pool::TestPool(conn);
        let app_state = super::AppState::new(pool, server_id::test()).unwrap();
        let mut server = test::TestServer::with_factory(move || app_router(app_state.clone()).unwrap());
        let request = server
            .client(Method::POST, "/log")
            .content_type("application/json")
            .body("{}")
            .unwrap();
        let response = server.execute(request.send()).unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let v: serde_json::Value = server.execute(response.json()).unwrap();

        let uuid = {
            let re = Regex::new(r"/log/([a-z0-9-]+)").unwrap();
            let url = response.headers()[header::LOCATION].to_str().unwrap();
            assert!(url.starts_with("/log/"));
            String::from(re.captures(&url).unwrap().get(1).unwrap().as_str())
        };
        assert!(v.is_object());
        assert_eq!(v["id"].as_str().unwrap(), uuid);
    }
}
