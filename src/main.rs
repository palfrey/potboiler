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
use std::io::Read;
use iron::modifiers::Redirect;

use std::env;
use std::ops::Deref;

extern crate uuid;
use uuid::Uuid;
extern crate serde_json;
use serde_json::Value;

extern crate r2d2;
extern crate r2d2_postgres;
extern crate persistent;
use persistent::Read as PRead;

#[macro_use] mod db;
#[macro_use] mod server_id;

use std::error::Error;
use std::fmt::{self, Debug};
#[derive(Debug)]
struct StringError(String);

impl Error for StringError {
    fn description(&self) -> &str { &*self.0 }
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

struct Log {
    id: Uuid,
    owner: Uuid,
    next: Option<Uuid>,
    prev: Option<Uuid>,
    data: Value
}

fn log_status(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Hello World!")))
}

fn new_log(req: &mut Request) -> IronResult<Response> {
    let conn = get_pg_connection!(&req);
    let body_string = {
        let mut body = String::new();
        req.body.read_to_string(&mut body).expect("could read from body");
        body
    };
    let json: Value = match serde_json::de::from_str(&body_string) {
        Ok(val) => val,
        Err(err) => return Err(IronError::new(err, (status::BadRequest, "Bad JSON")))
    };
    let id = Uuid::new_v4();
    let hyphenated = id.hyphenated().to_string();
    let server_id = get_server_id!(&req);
    conn.execute("INSERT INTO log (id, owner, data) VALUES ($1, $2, $3)",
                 &[&id, server_id.deref(), &json]).expect("insert worked");
    let new_url = {
        let req_url = req.url.clone();
        let base_url = req_url.into_generic_url();
        base_url.join(&format!("/log/{}", &hyphenated)).expect("join url works")
    };
    Ok(Response::with((status::Found, Redirect(iron::Url::from_generic_url(new_url).expect("URL parsed ok")))))
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
    chain.link_before(PRead::<server_id::ServerId>::one(server_id::setup()));
    chain.link(PRead::<db::PostgresDB>::both(pool));
    Iron::new(chain).http("localhost:8000").unwrap();
}
