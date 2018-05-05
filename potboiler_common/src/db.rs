use iron::typemap::Key;
use uuid::Uuid;
use serde_json;
use std::iter;
use deuterium;
use r2d2;
use r2d2_postgres;
use std::ops::Deref;
use postgres;
use std::marker;

error_chain! {
    errors {
        UniqueViolation
    }
    foreign_links {
        R2D2Error(r2d2::Error);
        PostgresError(postgres::Error);
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

#[derive(Debug)]
pub struct TestRow;

pub enum Row<'a> {
    Postgres(postgres::rows::Row<'a>),
    Test(&'a TestRow)
}
impl<'a> Row<'a> {
    pub fn get<T, R>(&self, id: R) -> T where T: FromSql, R: RowIndex {
        unimplemented!();
    }
    pub fn get_opt<T, R>(&self, id: R) -> Option<Result<T>> where T: FromSql, R: RowIndex {
        unimplemented!();
    }
}

pub struct TestRowIterator<'a> {
    rows: &'a Vec<TestRow>,
    location: usize
}

impl<'a> TestRowIterator<'a> {
    fn new(r: &'a Vec<TestRow>) -> TestRowIterator<'a> {
        TestRowIterator {
            rows: r,
            location: 0
        }
    }

    fn next(&mut self) -> Option<&'a TestRow> {
        self.location +=1;
        if self.location < self.rows.len() {
            Some(&self.rows[self.location])
        }
        else {
            None
        }
    }
}

pub enum RowIterator<'a> {
    Postgres(postgres::rows::Iter<'a>),
    Test(TestRowIterator<'a>)
}

impl<'a> Iterator for RowIterator<'a> {
    type Item = Row<'a>;

    fn next(&mut self) -> Option<Row<'a>> {
        match self {
            &mut RowIterator::Postgres(ref mut rows) => {
                rows.next().map(|r| Row::Postgres(r))
            }
            &mut RowIterator::Test(ref mut rows) => {
                rows.next().map(|r| Row::Test(r))
            }
        }
    }
}

pub enum Rows {
    Postgres(postgres::rows::Rows),
    Test(Vec<TestRow>)
    //_marker: &'stmt str
}
impl<'stmt> Rows {
    // fn new() -> Rows<'stmt>
    // {
    //     Rows{ _marker: "" }
    // }

    pub fn get<'a>(&self, id: i32) -> Row<'a> {
        unimplemented!();
    }
    pub fn is_empty(&self) -> bool {
        unimplemented!();
    }
    pub fn iter<'a>(&'a self) -> RowIterator<'a> {
        match self {
            &Rows::Postgres(ref rows) => {
                RowIterator::Postgres(rows.iter())
            }
            &Rows::Test(ref rows) => {
                RowIterator::Test(TestRowIterator::new(rows))
            }
        }
    }
}

impl<'a> iter::IntoIterator for &'a Rows {
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

#[derive(Debug, Clone)]
pub struct TestConnection;

impl TestConnection {
    fn get_rows(&self) -> Vec<TestRow> {
        vec!()
    }
}

#[derive(Debug)]
pub enum Connection {
    Postgres(r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>),
    Test(TestConnection)
}
impl<'conn> Connection {
    pub fn query(&'conn self, query: &str) -> Result<Rows> {
        match self {
            &Connection::Postgres(ref conn) => {
                Ok(Rows::Postgres(conn.query(query, &[])?))
            }
            &Connection::Test(ref conn) => {
                Ok(Rows::Test(conn.get_rows()))
            }
        }
    }
    pub fn dquery(&'conn self, squery: &deuterium::QueryToSql) -> Result<Rows> {
        self.query(&squery.to_final_sql(&mut deuterium::SqlContext::new(Box::new(deuterium::sql::PostgreSqlAdapter))))
    }
    pub fn execute(&self, equery: &str) -> Result<u64> {
        unimplemented!();
    }
    pub fn dexecute(&self, equery: &deuterium::QueryToSql) -> Result<u64> {
        self.execute(&equery.to_final_sql(&mut deuterium::SqlContext::new(Box::new(deuterium::sql::PostgreSqlAdapter))))
    }
}

#[derive(Clone, Debug)]
pub enum Pool {
    Postgres(r2d2::Pool<r2d2_postgres::PostgresConnectionManager>),
    TestPool(TestConnection)
}

impl Pool {
    pub fn get(&self) -> Result<Connection> {
        match self {
            &Pool::Postgres(ref pool) => {
                let conn = pool.get()?;
                Ok(Connection::Postgres(conn))
            }
            &Pool::TestPool(ref conn) => {
                Ok(Connection::Test(conn.clone()))
            }
        }
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
        Some(pool) => match pool.get() {
            Ok(conn) => {
                conn
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
