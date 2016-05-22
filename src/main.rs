#[macro_use] extern crate schemamama;
extern crate schemamama_postgres;
extern crate postgres;

use std::env;

mod schema;

fn main() {
    let db_url: &str = &env::var("DATABASE_URL").unwrap();
    let conn = postgres::Connection::connect(db_url, postgres::SslMode::None).unwrap();
    schema::up(&conn);
}
