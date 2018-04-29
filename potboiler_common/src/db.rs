use iron::typemap::Key;
use uuid::Uuid;
use serde_json;
use std::iter;
use std::marker;
use deuterium;
use r2d2;
use r2d2_postgres;

error_chain! {
    errors {
        UniqueViolation
    }
}

pub trait FromSql {}
impl FromSql for Uuid {}
impl FromSql for String {}
impl <'a> FromSql for &'a str {}
impl FromSql for Vec<u8> {}
impl FromSql for serde_json::Value {}

pub trait RowIndex {}
impl RowIndex for String {}
impl <'a> RowIndex for &'a str {}

pub struct Row<'a> {
    _marker: &'a str
}
impl<'a> Row<'a> {
    pub fn get<T, R>(&self, id: R) -> T where T: FromSql, R: RowIndex {
        unimplemented!();
    }
    pub fn get_opt<T, R>(&self, id: R) -> Option<Result<T>> where T: FromSql, R: RowIndex {
        unimplemented!();
    }
}

pub struct RowIterator<'a> {
    _marker: &'a str
}

impl<'a> Iterator for RowIterator<'a> {
    type Item = Row<'a>;

    fn next(&mut self) -> Option<Row<'a>> {
        unimplemented!();
    }
}

pub struct Rows<'stmt> {
    _marker: &'stmt str
}
impl<'stmt> Rows<'stmt> {
    pub fn get<'a>(&self, id: i32) -> Row<'a> {
        unimplemented!();
    }
    pub fn is_empty(&self) -> bool {
        unimplemented!();
    }
    pub fn iter<'a>(&'a self) -> RowIterator<'a> {
        unimplemented!();
    }
}

impl<'a> iter::IntoIterator for &'a Rows<'a> {
    type Item = Row<'a>;
    type IntoIter = RowIterator<'a>;
    fn into_iter(self) -> RowIterator<'a> {
        self.iter()
    }
}

pub struct Statement;
impl Statement {
    pub fn query(&self, query: &str) -> Result<Rows> {
        unimplemented!();
    }
}

pub struct Connection;
impl Connection {
    pub fn query(&self, query: &str) -> Result<Rows> {
        unimplemented!();
    }
    pub fn dquery(&self, squery: &deuterium::QueryToSql) -> Result<Rows> {
        self.query(&squery.to_final_sql(&mut deuterium::SqlContext::new(Box::new(deuterium::sql::PostgreSqlAdapter))))
    }
    pub fn execute(&self, equery: &str) -> Result<u64> {
        unimplemented!();
    }
    pub fn dexecute(&self, equery: &deuterium::QueryToSql) -> Result<u64> {
        self.execute(&equery.to_final_sql(&mut deuterium::SqlContext::new(Box::new(deuterium::sql::PostgreSqlAdapter))))
    }
}

pub struct ManageConnection;
impl ManageConnection {
    pub fn connect(&self) -> Result<Connection> {
        unimplemented!();
    }
}

#[derive(Clone, Debug)]
pub enum Pool {
    Postgres(r2d2::Pool<r2d2_postgres::PostgresConnectionManager>),
    TestPool
}

impl Pool {
    pub fn get(&self) -> Result<Box<ManageConnection>> {
        unimplemented!();
    }
}

pub struct PoolKey;

impl Key for PoolKey {
    type Value = Pool;
}

// Gets a connection from the pool from the given request or returns a 500
#[macro_export]
macro_rules! get_db_connection {
    ($req:expr) => (match $req.extensions.get::<persistent::Read<db::PoolKey>>() {
        Some(pool) => match pool.deref().get() {
            Ok(conn) => {
                conn.connect().unwrap()
            }
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
