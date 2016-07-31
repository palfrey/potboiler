use iron::typemap::Key;
use r2d2;
use r2d2_postgres::{PostgresConnectionManager, SslMode};

pub type PostgresPool = r2d2::Pool<PostgresConnectionManager>;
pub type PostgresConnection = r2d2::PooledConnection<PostgresConnectionManager>;

pub struct PostgresDB;
impl Key for PostgresDB { type Value = PostgresPool;}

// Gets a connection from the pool from the given request or returns a 500
macro_rules! get_pg_connection {
    ($req:expr) => (match $req.extensions.get::<persistent::Read<db::PostgresDB>>() {
        Some(pool) => match pool.get() {
            Ok(conn) => conn,
            Err(_) => {
                println!("Couldn't get a connection to pg!");
                return Ok(Response::with((status::InternalServerError)));
            }
        },
        None => {
            println!("Couldn't get the pg pool from the request!");
            return Ok(Response::with((status::InternalServerError)));
        }
    })
}

pub fn get_pool(uri: &str) -> PostgresPool {
    let manager = PostgresConnectionManager::new(uri, SslMode::None).expect("Needed a working DATABASE_URL");
    r2d2::Pool::new(r2d2::Config::default(), manager).unwrap()
}
