//! This crate provide a SQlite engine to read and work with SQlite database files.
//!
//! This is a work in progress and a learning project.
//!
//! TODO: redo this doc, there are three components: db executor and parser
//!
//! The main component is [db] module that allows to run dot command or execute sql queries.
//!
//! The [db] module contains:
//! * [DB](db::DB) struct that represents the database and provide methods to interact with it.
//! * [dbmetadata](db::dbmetadata) module that contains types to represent the database metadata.
//! * [table](db::table) module that contains types to represent database tables.
//! * [db_response](db::db_response) module that contains types to represent the response of a
//!
//! The rest is mainly focused on parsing the database file format. It's not part of the public
//! API.
//! The [fileformat](db::fileformat) module contains:
//! - [page](db::fileformat::page) module that contains types to represent database pages.
//! - [record](db::fileformat::record) module that contains types to represent database records.
//! - [types](db::fileformat::types) module that contains types to represent low level types used
//!
//!The [parser](parser) module contains:
//! - [parser](parser) module that contains the main parser to parse queries.
//! - [statement](parser::statement) module that contains types to represent parsed queries.
//! - [tokenizer](parser::tokenizer) module that contains the tokenizer to parse queries.
//!
//! # Example
//! ```
//! use codecrafters_sqlite::db::DB;
//! let mut db = DB::new("test.db").unwrap();
//! let response = db.execute("SELECT name FROM users;").unwrap();
//! db.execute(".tables").unwrap();
//!```
//!
//! # The execute command response
//! The execute returns a Result<Option<Vec<(Query, Response)>>
//! For dot commands, the Option will be None.
//! For sql queries, the Option will be Some with a vector of (Query, Response) tuples.
//!
//! A [Statement](parser::statement::Statement) represents the parsed query.
//! A [Response](db::db_response::Response) represents the result of executing the query which is a
//! vector of rows. Each rows is a vector of [RType](db::db_response::RType).

pub mod db;
pub mod executor;
pub mod parser;
