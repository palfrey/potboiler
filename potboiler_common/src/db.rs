use iron::typemap::Key;
use r2d2;
use r2d2_diesel;
use diesel;
use std::fmt;
use std::result;
use diesel::sql_types;
use byteorder;
use std;

error_chain! {
    errors {
        UniqueViolation
    }
    foreign_links {
        R2D2Error(r2d2::Error);
    }
}

pub struct TypeMetadata;
pub struct MetadataLookup;
pub struct QueryBuilder;
pub struct BindCollector;
pub struct RawValue;

#[derive(Copy, Clone, Debug, PartialOrd, Hash)]
pub struct ByteOrder;
pub struct Backend;

impl std::cmp::Ord for ByteOrder{}
impl byteorder::ByteOrder for ByteOrder{}

impl diesel::query_builder::QueryBuilder<Backend> for QueryBuilder {
    fn push_sql(&mut self, sql: &str) {
        unimplemented!()
    }
    fn push_identifier(&mut self, identifier: &str) -> diesel::QueryResult<()> {
        unimplemented!()
    }
    fn push_bind_param(&mut self) {
        unimplemented!()
    }
    fn finish(self) -> String {
        unimplemented!()
    }
}

impl diesel::backend::Backend for Backend {
    type QueryBuilder = QueryBuilder;
    type BindCollector = BindCollector;
    type RawValue = RawValue;
    type ByteOrder = ByteOrder;
}

impl diesel::backend::TypeMetadata for Backend {
    type TypeMetadata = TypeMetadata;
    type MetadataLookup = MetadataLookup;
}

macro_rules! backend_sql_type {
    ($kind:ty) => (impl sql_types::HasSqlType<$kind> for Backend {
        fn metadata(lookup: &Self::MetadataLookup) -> Self::TypeMetadata
        {
            unimplemented!();
        }

        fn row_metadata(
            out: &mut Vec<Self::TypeMetadata>,
            lookup: &Self::MetadataLookup
        ) {
            unimplemented!();
        }
    })
}
backend_sql_type!(sql_types::SmallInt);
backend_sql_type!(sql_types::Integer);
backend_sql_type!(sql_types::BigInt);
backend_sql_type!(sql_types::Float);
backend_sql_type!(sql_types::Double);
backend_sql_type!(sql_types::Text);
backend_sql_type!(sql_types::Binary);
backend_sql_type!(sql_types::Date);
backend_sql_type!(sql_types::Time);
backend_sql_type!(sql_types::Timestamp);

pub enum Connection {
    Postgres(r2d2::Pool<r2d2_diesel::ConnectionManager<diesel::pg::PgConnection>>),
    Test
}

impl diesel::connection::SimpleConnection for Connection {
    fn batch_execute(&self, query: &str) -> diesel::QueryResult<()> {
        unimplemented!();
    }
}

impl diesel::Connection for Connection {
    type Backend = Backend;
    fn establish(database_url: &str) -> diesel::ConnectionResult<Self> {
        unimplemented!();
    }

    fn transaction<T, E, F>(&self, f: F) -> result::Result<T, E>
    where
        F: FnOnce() -> result::Result<T, E>,
        E: From<diesel::result::Error>,
    {
        unimplemented!();
    }

    fn begin_test_transaction(&self) -> diesel::QueryResult<()> {
        unimplemented!();
    }

    fn test_transaction<T, E, F>(&self, f: F) -> T
    where
        F: FnOnce() -> result::Result<T, E>,
        E: fmt::Debug {
        unimplemented!();
    }
}

// impl Connection {
//     fn get<C>(&self) -> Result<C> where C: diesel::Connection {
//         match self {
//             &Connection::Postgres(pool) => {
//                 Ok(pool.get()?.deref())
//             }
//             _ => {
//                 unimplemented!();
//             }
//         }
//     }
// }

pub struct ConnectionKey;
impl Key for ConnectionKey {
    type Value = Connection;
}

// Gets a connection from the pool from the given request or returns a 500
#[macro_export]
macro_rules! get_db_connection {
    ($req:expr) => (match $req.extensions.get::<persistent::Read<db::ConnectionKey>>() {
        Some(pool) => pool.deref(),
        None => {
            println!("Couldn't get the db pool from the request!");
            return Ok(Response::with((status::InternalServerError)));
        }
    })
}
