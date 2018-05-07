use iron::typemap::Key;
use uuid::Uuid;
use serde_json;
use std::iter;
use r2d2;
use r2d2_postgres;
use postgres;
use std::convert::From;
use std::collections::HashMap;
use std::fmt;
use regex;

error_chain! {
    errors {
        UniqueViolation
        NoTestQuery(cmd: String)
        NoTestExecute(cmd: String)
        PostgresError(cmd: String)
    }
    foreign_links {
        R2D2Error(r2d2::Error);
    }
}

pub struct HexSlice<'a>(&'a [u8]);

impl<'a> HexSlice<'a> {
    pub fn new<T>(data: &'a T) -> HexSlice<'a>
        where T: ?Sized + AsRef<[u8]> + 'a
    {
        HexSlice(data.as_ref())
    }
}

impl<'a> fmt::Display for HexSlice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{:02X}", byte)?;
        }
        Ok(())
    }
}

pub trait FromSql {}
impl FromSql for u32 {}
impl FromSql for Uuid {}
impl FromSql for String {}
impl <'a> FromSql for &'a str {}
impl FromSql for Vec<u8> {}
impl FromSql for serde_json::Value {}

#[derive(Debug, Clone)]
pub enum SqlValue {
    U32(u32),
    UUID(Uuid),
    String(String),
    U8Bytes(Vec<u8>),
    JSON(serde_json::Value),
}

impl From<u32> for SqlValue {
    fn from(i: u32) -> SqlValue { SqlValue::U32(i) }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum ValueIndex
{
    U32(u32),
    String(String),
}

impl<'a> From<&'a str> for ValueIndex {
    fn from(s: &str) -> ValueIndex { ValueIndex::String(String::from(s))}
}

pub trait RowIndex {
    fn val(&self) -> ValueIndex;
}
impl RowIndex for u32 {
    fn val(&self) -> ValueIndex { ValueIndex::U32(*self)}
}
impl RowIndex for String {
    fn val(&self) -> ValueIndex { ValueIndex::String(self.clone())}
}
impl <'a> RowIndex for &'a str {
    fn val(&self) -> ValueIndex { ValueIndex::String(String::from(*self))}
}

#[derive(Debug, Clone)]
pub struct TestRow
{
    data: HashMap<ValueIndex, SqlValue>
}

pub trait GetRow<T> {
    fn get<R>(&self, id: R) -> T where T: FromSql, R: RowIndex + fmt::Display;
}

impl TestRow {
    pub fn new() -> TestRow {
        TestRow{data: HashMap::new()}
    }

    pub fn insert<K, V>(&mut self, k: K, v: V) -> Option<SqlValue>
        where K: Into<ValueIndex>, V: Into<SqlValue> {
        self.data.insert(k.into(), v.into())
    }
}

macro_rules! get_row {
    ($type: ty, $kind: path) => (impl<'a> GetRow<$type> for TestRow {
    fn get<R>(&self, id: R) -> $type where R: RowIndex + fmt::Display {
        if !self.data.contains_key(&id.val()) {
            panic!(format!("Can't find key {} in row", id));
        }
        match self.data[&id.val()] {
            $kind(ref val) => val.clone(),
            _ => panic!()
        }
    }
})}

get_row!(u32, SqlValue::U32);
get_row!(Uuid, SqlValue::UUID);
get_row!(String, SqlValue::String);
get_row!(serde_json::Value, SqlValue::JSON);
get_row!(Vec<u8>, SqlValue::U8Bytes);

pub enum Row<'a> {
    Postgres(postgres::rows::Row<'a>),
    Test(&'a TestRow)
}
impl<'a> Row<'a> {
    pub fn get<T, R>(&self, id: R) -> T 
    where
        T: FromSql + postgres::types::FromSql, 
        R: RowIndex + postgres::rows::RowIndex + fmt::Display + fmt::Debug,
        TestRow: GetRow<T> {
        match self {
            &Row::Postgres(ref rows) => {
                rows.get(id)
            }
            &Row::Test(ref rows) => {
                rows.get(id)
            }
        }
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
}

impl<'stmt> Rows {
    pub fn get<'a>(&'a self, id: usize) -> Row<'a> {
        match self {
            &Rows::Postgres(ref rows) => {
                Row::Postgres(rows.get(id))
            }
            &Rows::Test(ref rows) => {
                Row::Test(rows.get(id).unwrap())
            }
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            &Rows::Postgres(ref rows) => {
                rows.is_empty()
            }
            &Rows::Test(ref rows) => {
                rows.is_empty()
            }
        }
    }
    pub fn len(&self) -> usize {
        match self {
            &Rows::Postgres(ref rows) => {
                rows.len()
            }
            &Rows::Test(ref rows) => {
                rows.len()
            }
        }
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

#[derive(Debug, Clone)]
pub struct TestConnection {
    query_results: Vec<(regex::Regex, Vec<TestRow>)>,
    execute_results: Vec<(regex::Regex, u64)>
}

impl TestConnection {
    pub fn new() -> TestConnection {
        TestConnection {
            query_results: Vec::new(),
            execute_results: Vec::new()
        }
    }

    pub fn add_test_query(&mut self, cmd: &str, results: Vec<TestRow>) {
        self.query_results.push((regex::Regex::new(cmd).unwrap(), results));
    }

    pub fn add_test_execute(&mut self, cmd: &str, results: u64) {
        self.execute_results.push((regex::Regex::new(cmd).unwrap(), results));
    }

    fn get_rows(&self, cmd: &str) -> Result<Vec<TestRow>> {
        for &(ref patt, ref res) in self.query_results.iter() {
            if patt.is_match(cmd) {
                return Ok(res.clone());
            }
        }
        Err(Error::from(ErrorKind::NoTestQuery(String::from(cmd))))
    }
    fn execute(&self, cmd: &str) -> Result<u64> {
        for &(ref patt, res) in self.execute_results.iter() {
            if patt.is_match(cmd) {
                return Ok(res);
            }
        }
        Err(Error::from(ErrorKind::NoTestExecute(String::from(cmd))))
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
                Ok(Rows::Postgres(conn.query(query, &[]).map_err(|e| Error::with_chain(e, ErrorKind::PostgresError(query.to_string())))?))
            }
            &Connection::Test(ref conn) => {
                Ok(Rows::Test(conn.get_rows(query)?))
            }
        }
    }
    pub fn execute(&self, equery: &str) -> Result<u64> {
        match self {
            &Connection::Postgres(ref conn) => {
                conn.execute(equery, &[]).map_err(|e| Error::with_chain(e, ErrorKind::PostgresError(equery.to_string())))
            }
            &Connection::Test(ref conn) => {
                conn.execute(equery)
            }
        }
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
                return Ok(Response::with(status::InternalServerError));
            }
        },
        None => {
            println!("Couldn't get the pg pool from the request!");
            return Ok(Response::with(status::InternalServerError));
        }
    })
}
