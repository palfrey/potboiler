use crate::db;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};

pub type PostgresPool = r2d2::Pool<PostgresConnectionManager>;
pub type PostgresConnection = r2d2::PooledConnection<PostgresConnectionManager>;

pub fn get_pool(uri: &str) -> Result<db::Pool, db::Error> {
    let manager = PostgresConnectionManager::new(uri, TlsMode::None).expect("Needed a working DATABASE_URL");
    Ok(db::Pool::Postgres(
        r2d2::Pool::builder()
            .build(manager)
            .map_err(|e| db::Error::R2D2Error { cause: e.to_string() })?,
    ))
}
