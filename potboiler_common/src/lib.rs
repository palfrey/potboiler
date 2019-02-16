#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    warnings,
    trivial_numeric_casts,
    unstable_features,
    unused,
    future_incompatible
)]

pub mod clock;
pub mod db;
pub mod pg;
pub mod server_id;
pub mod types;

use hybrid_clocks::{Timestamp, WallT};

pub fn get_raw_timestamp(timestamp: &Timestamp<WallT>) -> Result<db::HexSlice, ::std::io::Error> {
    let mut raw_timestamp: Vec<u8> = Vec::new();
    timestamp.write_bytes(&mut raw_timestamp)?;
    Ok(db::HexSlice::new(raw_timestamp))
}

#[macro_export]
macro_rules! iron_error_from {
    () => {
        impl From<ErrorKind> for IronError {
            fn from(errkind: ErrorKind) -> IronError {
                let desc = format!("{:?}", errkind);
                return IronError::new(Error::from_kind(errkind), (status::BadRequest, desc));
            }
        }

        impl From<Error> for IronError {
            fn from(error: Error) -> IronError {
                let desc = format!("{:?}", error);
                return IronError::new(error, (status::BadRequest, desc));
            }
        }
    };
}

#[cfg(test)]
mod test {
    #[test]
    fn test_select() {
        let mut conn = super::db::TestConnection::new();
        let mut row = super::db::TestRow::new();
        row.insert("id", 1);
        conn.add_test_query("select 1 as id from test", vec![row]);
        let pool = super::db::Pool::TestPool(conn);
        let rows = pool.get().unwrap().query("select 1 as id from test").unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows.get(0).get::<u32, &str>("id"), 1);
    }

    #[test]
    fn test_insert() {
        let mut conn = super::db::TestConnection::new();
        conn.add_test_execute(r"insert into test \(id\) values\(1\)", 1);
        let pool = super::db::Pool::TestPool(conn);
        let res = pool.get().unwrap().execute("insert into test (id) values(1)").unwrap();
        assert_eq!(res, 1);
    }
}
