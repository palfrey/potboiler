use r2d2;
use db;
use diesel::PgConnection;
use r2d2_diesel::ConnectionManager;

pub fn get_pool(uri: &str) -> Result<db::Connection, r2d2::Error> {
    let manager = ConnectionManager::<PgConnection>::new(uri);
    let pool = r2d2::Pool::builder().build(manager)?;
    Ok(db::Connection::Postgres(pool))
}