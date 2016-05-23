#[macro_use] extern crate schemamama;
extern crate schemamama_postgres;
extern crate postgres;
mod schema;

#[macro_use] extern crate log;
extern crate log4rs;

extern crate pencil;
use pencil::{Pencil, Request, Response, PencilResult};
//use pencil::http_errors::BadRequest;

use std::env;

fn log_status(request: &mut Request) -> PencilResult {
    info!("{:?}", request);
    Ok(Response::from("Hello World!"))
}

fn main() {
    log4rs::init_file("log.yaml", Default::default()).unwrap();
    let db_url: &str = &env::var("DATABASE_URL").expect("Needed DATABASE_URL");
    let conn = postgres::Connection::connect(db_url, postgres::SslMode::None).expect("Needed a working DATABASE_URL");
    schema::up(&conn).unwrap();
    let mut app = Pencil::new("");
    app.set_debug(true);
    app.set_log_level();
    app.get("/log", "log status", log_status);
    app.post("/log", "add new local log entry", log_status);
    app.get("/log/<entry_id:string>", "get log with UUID", log_status);
    app.run("127.0.0.1:8000");
}
