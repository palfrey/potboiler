use postgres::error::Error as PError;
use serde_json;
use std::error::Error;
use std::fmt::{self, Debug};
use std::io;
use uuid;

#[derive(Debug)]
pub struct StringError(pub String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl From<String> for StringError {
    fn from(msg: String) -> StringError {
        StringError(msg)
    }
}

impl<'a> From<&'a str> for StringError {
    fn from(msg: &str) -> StringError {
        StringError(String::from(msg))
    }
}

impl From<io::Error> for StringError {
    fn from(error: io::Error) -> StringError {
        StringError::from(error.description())
    }
}

impl From<serde_json::Error> for StringError {
    fn from(se: serde_json::Error) -> StringError {
        StringError::from(se.description())
    }
}

impl From<uuid::ParseError> for StringError {
    fn from(ue: uuid::ParseError) -> StringError {
        StringError::from(ue.description())
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
