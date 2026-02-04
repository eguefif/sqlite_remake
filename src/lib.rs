//! This crate provide a SQlite engine to read and work with SQlite database files.
//!
//! This is a work in progress and a learning project.
//!
//!
//! There are three components:
//! The [executor] module is responsible to take a query and return a response.
//! The [db] module contains the sqlite file representation.
//! The [parser] is used by the [executor] to get a query object.
//!
//! The [db] module contains:
//! * [DB](db::DB) struct that represents the database and provide methods to interact with it.
//! * [dbmetadata](db::dbmetadata) module that contains types to represent the database metadata.
//! * [table](db::table) module that contains types to represent database tables.
//! * [db_response](executor::db_response) module that contains types to represent the response of a
//!
//!The [parser] module contains:
//! - [parser] module that contains the main parser to parse queries.
//! - [statement](parser::statement) module that contains types to represent parsed queries.
//! - [tokenizer](parser::tokenizer) module that contains the tokenizer to parse queries.
//!
//! The [Executor](executor)
//!
//! # Example
//! ```no_run
//!    fn main() {
//!    let db = match DB::new("sample.db") {
//!        Ok(db) => db,
//!        Err(error) => eprintln!("Impossible to read database metadata: {}", error),
//!    };
//!    let mut executor = Executor::new(db);
//!    if let Some(response) = executor.execute(command).unwrap() {
//!        display_response(&response);
//!    }
//!    }
//!```

pub mod db;
pub mod executor;
pub mod parser;
