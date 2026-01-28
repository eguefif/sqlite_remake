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
use crate::db::fileformat::record::FieldType;
use std::fmt::{Display, Formatter, Result};

pub enum RType {
    Num(i64),
    Blob(Vec<u8>),
    Str(String),
    Null,
}

impl RType {
    pub fn from_fieldtype(field: FieldType) -> RType {
        match field {
            FieldType::TNull => RType::Null,
            FieldType::TI8(value) => RType::Num(value as i64),
            FieldType::TI16(value) => RType::Num(value as i64),
            FieldType::TI32(value) => RType::Num(value as i64),
            FieldType::TI48(value) => RType::Num(value as i64),
            FieldType::TI64(value) => RType::Num(value as i64),
            FieldType::TF64(value) => RType::Num(value as i64),
            FieldType::T0 => RType::Num(0),
            FieldType::T1 => RType::Num(1),
            FieldType::TVar => panic!("Variable type field in record while getting response"),
            FieldType::TBlob(blob) => RType::Blob(blob),
            FieldType::TStr(string) => RType::Str(string.clone()),
        }
    }
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
