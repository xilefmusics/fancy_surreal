use std::error;
use std::fmt;
use surrealdb;

#[derive(Debug)]
pub enum Error {
    Surreal(String),
    FancySurreal(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Surreal(msg) => write!(f, "Surreal({})", msg),
            Self::FancySurreal(msg) => write!(f, "FancySurreal({})", msg),
        }
    }
}

impl error::Error for Error {}

impl From<surrealdb::Error> for Error {
    fn from(err: surrealdb::Error) -> Self {
        Self::Surreal(err.to_string())
    }
}

impl Error {
    pub fn new(msg: &str) -> Self {
        Self::FancySurreal(msg.to_string())
    }
}
