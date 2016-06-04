#[macro_use] extern crate schemamama;
extern crate schemamama_postgres;
extern crate postgres;
mod schema;

#[macro_use] extern crate log;
extern crate log4rs;

extern crate iron;
extern crate router;

use iron::prelude::*;
use iron::status;
use router::Router;

use std::env;

fn log_status(request: &mut Request) -> IronResult<Response> {
    info!("{:?}", request);
    Ok(Response::with((status::Ok, "Hello World!")))
}

fn main() {
    log4rs::init_file("log.yaml", Default::default()).unwrap();
    let db_url: &str = &env::var("DATABASE_URL").expect("Needed DATABASE_URL");
    let conn = postgres::Connection::connect(db_url, postgres::SslMode::None).expect("Needed a working DATABASE_URL");
    schema::up(&conn).unwrap();
    let mut router = Router::new();
    router.get("/log", log_status);
    router.post("/log", log_status);
    router.get("/log/:entry_id", log_status);
    Iron::new(router).http("localhost:8000").unwrap();
}
