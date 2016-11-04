use postgres::error::Error as PError;
use std::error::Error;
use std::fmt::{self, Debug};

#[derive(Debug)]
pub struct StringError(pub String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl<'a> From<&'a str> for StringError {
    fn from(mesg: &str) -> StringError {
        StringError(String::from(mesg))
    }
}

impl From<PError> for StringError {
    fn from(pe: PError) -> StringError {
        StringError::from(pe.description())
    }
}

impl Error for StringError {
    fn description(&self) -> &str {
        &*self.0
    }
}
