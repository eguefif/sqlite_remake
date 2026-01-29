//! Module for handling database response types.
//!
//! # How to read response
//! The response is represented as a vector of rows, where each row is a vector of RType.
//!
//! # Example
//! ```no_run
//! use codecrafters_sqlite::db::db_response::{RType, Response};
//! let db = codecrafters_sqlite::db::DB::new("test.db").unwrap();
//! let response = db.execute("SELECT name, age, photo FROM users;").unwrap();
//! if let Some(responses) = response {
//!    for (_query, response) in responses {
//!         for row in response {
//!             for col in row {
//!                 println!("{}", col);
//!             }
//!         }
//!    }
//! }
//! ```
//!
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum RType {
    Num(i64),
    Blob(Vec<u8>),
    Str(String),
    Null,
}

impl Display for RType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            RType::Num(value) => write!(f, "{}", value),
            RType::Blob(value) => write!(f, "{:?}", value),
            RType::Str(value) => write!(f, "{}", value),
            RType::Null => write!(f, "Null"),
        }
    }
}

pub type Response = Vec<Vec<RType>>;
