use r2d2;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use db;

pub type PostgresPool = r2d2::Pool<PostgresConnectionManager>;
pub type PostgresConnection = r2d2::PooledConnection<PostgresConnectionManager>;

pub fn get_pool(uri: &str) -> Result<db::Pool, r2d2::Error> {
    let manager = PostgresConnectionManager::new(uri, TlsMode::None).expect("Needed a working DATABASE_URL");
    Ok(db::Pool::Postgres(r2d2::Pool::builder()
        .build(manager)?))
}