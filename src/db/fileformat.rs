//! Interal module for handling file format operations.
//! It's not part of the public API.
//!
//! It is based on SQlite documentation about file format:
//! [Sqlite fileformat doc](https://www.sqlite.org/fileformat.html)
//!
//! It contains three main modules:
//! * [page] A module that offer a way to read page in the db
//! * [record] A module that allows to read one record
//! * [types]  Type associated of the fileformat
pub mod page;
pub mod record;
pub mod types;
