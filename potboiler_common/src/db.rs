use error_chain::{
    // FIXME: Need https://github.com/rust-lang-nursery/error-chain/pull/253
    error_chain,
    error_chain_processing,
    impl_error_chain_kind,
    impl_error_chain_processed,
    impl_extract_backtrace,
};
use postgres;
use postgres_shared;
use r2d2;
use r2d2_postgres;
use regex;
use serde_json;
use std::{collections::HashMap, convert::From, fmt, iter};
use uuid::Uuid;

error_chain! {
    errors {
        UniqueViolation
        NoTestQuery(cmd: String)
        NoTestExecute(cmd: String)
        PostgresError(cmd: String)
        NoSuchTable
    }
    foreign_links {
        R2D2Error(r2d2::Error);
    }
}

impl Clone for Error {
    fn clone(&self) -> Error {
        match self {
            Error(ErrorKind::PostgresError(cmd), _) => Error::from(ErrorKind::PostgresError(cmd.clone())),
            Error(ErrorKind::NoSuchTable, _) => Error::from(ErrorKind::NoSuchTable),
            _ => unimplemented!(),
        }
    }
}

unsafe impl Sync for Error {}

#[derive(Debug)]
pub struct HexSlice(Vec<u8>);

impl HexSlice {
    pub fn new(data: Vec<u8>) -> HexSlice {
        HexSlice(data)
    }

    pub fn sql(&self) -> String {
        format!("decode('{}', 'hex')", self)
    }
}

impl fmt::Display for HexSlice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for byte in self.0.iter() {
            write!(f, "{:02X}", byte)?;
        }
        Ok(())
    }
}

pub trait FromSql {}
impl<T: FromSql> FromSql for Option<T> {}
impl FromSql for u32 {}
impl FromSql for Uuid {}
impl FromSql for String {}
impl<'a> FromSql for &'a str {}
impl FromSql for Vec<u8> {}
impl FromSql for serde_json::Value {}

#[derive(Debug, Clone)]
pub enum SqlValue {
    Null,
    U32(u32),
    UUID(Uuid),
    String(String),
    U8Bytes(Vec<u8>),
    JSON(serde_json::Value),
}

impl From<u32> for SqlValue {
    fn from(i: u32) -> SqlValue {
        SqlValue::U32(i)
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum ValueIndex {
    U32(u32),
    String(String),
}

impl<'a> From<&'a str> for ValueIndex {
    fn from(s: &str) -> ValueIndex {
        ValueIndex::String(String::from(s))
    }
}

pub trait RowIndex {
    fn val(&self) -> ValueIndex;
}
impl RowIndex for u32 {
    fn val(&self) -> ValueIndex {
        ValueIndex::U32(*self)
    }
}
impl RowIndex for String {
    fn val(&self) -> ValueIndex {
        ValueIndex::String(self.clone())
    }
}
impl<'a> RowIndex for &'a str {
    fn val(&self) -> ValueIndex {
        ValueIndex::String(String::from(*self))
    }
}

#[derive(Debug, Clone, Default)]
pub struct TestRow {
    data: HashMap<ValueIndex, SqlValue>,
}

pub trait GetRow<T> {
    fn get<R>(&self, id: R) -> T
    where
        T: FromSql,
        R: RowIndex + fmt::Display;
}

impl TestRow {
    pub fn new() -> TestRow {
        TestRow { data: HashMap::new() }
    }

    pub fn insert<K, V>(&mut self, k: K, v: V) -> Option<SqlValue>
    where
        K: Into<ValueIndex>,
        V: Into<SqlValue>,
    {
        self.data.insert(k.into(), v.into())
    }
}

macro_rules! get_row {
    ($type: ty, $kind: path) => {
        impl GetRow<$type> for TestRow {
            fn get<R>(&self, id: R) -> $type
            where
                R: RowIndex + fmt::Display,
            {
                if !self.data.contains_key(&id.val()) {
                    panic!(format!("Can't find key {} in row", id));
                }
                match self.data[&id.val()] {
                    $kind(ref val) => val.clone(),
                    _ => panic!(),
                }
            }
        }

        impl GetRow<Option<$type>> for TestRow {
            fn get<R>(&self, id: R) -> Option<$type>
            where
                R: RowIndex + fmt::Display,
            {
                if !self.data.contains_key(&id.val()) {
                    panic!(format!("Can't find key {} in row", id));
                }
                match self.data[&id.val()] {
                    $kind(ref val) => Some(val.clone()),
                    SqlValue::Null => None,
                    _ => panic!(),
                }
            }
        }
    };
}

get_row!(u32, SqlValue::U32);
get_row!(Uuid, SqlValue::UUID);
get_row!(String, SqlValue::String);
get_row!(serde_json::Value, SqlValue::JSON);
get_row!(Vec<u8>, SqlValue::U8Bytes);

#[derive(Debug)]
pub enum Row<'a> {
    Postgres(postgres::rows::Row<'a>),
    Test(&'a TestRow),
}
impl<'a> Row<'a> {
    pub fn get<T, R>(&self, id: R) -> T
    where
        T: FromSql + postgres::types::FromSql,
        R: RowIndex + postgres::rows::RowIndex + fmt::Display + fmt::Debug,
        TestRow: GetRow<T>,
    {
        match *self {
            Row::Postgres(ref rows) => rows.get(id),
            Row::Test(ref rows) => rows.get(id),
        }
    }
    #[allow(clippy::needless_pass_by_value)]
    pub fn get_opt<T, R>(&self, _id: R) -> Option<Result<T>>
    where
        T: FromSql,
        R: RowIndex,
    {
        unimplemented!();
    }
}

#[derive(Debug)]
pub struct TestRowIterator<'a> {
    rows: &'a [TestRow],
    location: usize,
}

impl<'a> TestRowIterator<'a> {
    fn new(r: &'a [TestRow]) -> TestRowIterator<'a> {
        TestRowIterator { rows: r, location: 0 }
    }

    fn next(&mut self) -> Option<&'a TestRow> {
        self.location += 1;
        if self.location < self.rows.len() {
            Some(&self.rows[self.location])
        } else {
            None
        }
    }
}

pub enum RowIterator<'a> {
    Postgres(postgres::rows::Iter<'a>),
    Test(TestRowIterator<'a>),
}

impl<'a> fmt::Debug for RowIterator<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RowIterator::Postgres(_) => write!(f, "RowIterator(PostgresRows)"),
            RowIterator::Test(ref rows) => write!(f, "RowIterator({:?})", rows),
        }
    }
}

impl<'a> Iterator for RowIterator<'a> {
    type Item = Row<'a>;

    fn next(&mut self) -> Option<Row<'a>> {
        match *self {
            RowIterator::Postgres(ref mut rows) => rows.next().map(Row::Postgres),
            RowIterator::Test(ref mut rows) => rows.next().map(Row::Test),
        }
    }
}

#[derive(Debug)]
pub enum Rows {
    Postgres(postgres::rows::Rows),
    Test(Vec<TestRow>),
}

impl<'stmt> Rows {
    pub fn get(&self, id: usize) -> Row {
        match *self {
            Rows::Postgres(ref rows) => Row::Postgres(rows.get(id)),
            Rows::Test(ref rows) => Row::Test(&rows[id]),
        }
    }
    pub fn is_empty(&self) -> bool {
        match *self {
            Rows::Postgres(ref rows) => rows.is_empty(),
            Rows::Test(ref rows) => rows.is_empty(),
        }
    }
    pub fn len(&self) -> usize {
        match *self {
            Rows::Postgres(ref rows) => rows.len(),
            Rows::Test(ref rows) => rows.len(),
        }
    }
    pub fn iter(&self) -> RowIterator {
        match *self {
            Rows::Postgres(ref rows) => RowIterator::Postgres(rows.iter()),
            Rows::Test(ref rows) => RowIterator::Test(TestRowIterator::new(rows)),
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

#[derive(Debug, Clone, Default)]
pub struct TestConnection {
    query_results: Vec<(regex::Regex, Result<Vec<TestRow>>)>,
    execute_results: Vec<(regex::Regex, Result<u64>)>,
}

impl TestConnection {
    pub fn new() -> TestConnection {
        TestConnection {
            query_results: Vec::new(),
            execute_results: Vec::new(),
        }
    }

    pub fn add_test_query(&mut self, cmd: &str, results: Vec<TestRow>) {
        self.query_results.push((regex::Regex::new(cmd).unwrap(), Ok(results)));
    }

    pub fn add_failed_query(&mut self, cmd: &str, err: ErrorKind) {
        self.query_results
            .push((regex::Regex::new(cmd).unwrap(), Err(Error::from(err))));
    }

    pub fn add_test_execute(&mut self, cmd: &str, results: u64) {
        self.execute_results
            .push((regex::Regex::new(cmd).unwrap(), Ok(results)));
    }

    fn get_rows(&self, cmd: &str) -> Result<Vec<TestRow>> {
        for &(ref patt, ref res) in self.query_results.iter() {
            if patt.is_match(cmd) {
                return res.clone();
            }
        }
        Err(Error::from(ErrorKind::NoTestQuery(String::from(cmd))))
    }
    fn execute(&self, cmd: &str) -> Result<u64> {
        for &(ref patt, ref res) in self.execute_results.iter() {
            if patt.is_match(cmd) {
                return res.clone();
            }
        }
        Err(Error::from(ErrorKind::NoTestExecute(String::from(cmd))))
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Connection {
    Postgres(r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>),
    Test(TestConnection),
}

fn convert_postgres_error(e: postgres_shared::error::Error, query: &str) -> Error {
    let err = if *e.code().unwrap() == postgres_shared::error::UNIQUE_VIOLATION {
        ErrorKind::UniqueViolation
    } else {
        ErrorKind::PostgresError(query.to_string())
    };
    Error::with_chain(e, err)
}

impl<'conn> Connection {
    pub fn query(&'conn self, query: &str) -> Result<Rows> {
        match *self {
            Connection::Postgres(ref conn) => Ok(Rows::Postgres(
                conn.query(query, &[]).map_err(|e| convert_postgres_error(e, query))?,
            )),
            Connection::Test(ref conn) => Ok(Rows::Test(conn.get_rows(query)?)),
        }
    }
    pub fn execute(&self, equery: &str) -> Result<u64> {
        match *self {
            Connection::Postgres(ref conn) => conn.execute(equery, &[]).map_err(|e| convert_postgres_error(e, equery)),
            Connection::Test(ref conn) => conn.execute(equery),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Pool {
    Postgres(r2d2::Pool<r2d2_postgres::PostgresConnectionManager>),
    TestPool(TestConnection),
}

impl Pool {
    pub fn get(&self) -> Result<Connection> {
        match *self {
            Pool::Postgres(ref pool) => {
                let conn = pool.get()?;
                Ok(Connection::Postgres(conn))
            }
            Pool::TestPool(ref conn) => Ok(Connection::Test(conn.clone())),
        }
    }
}
