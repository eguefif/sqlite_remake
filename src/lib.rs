//! This crate provide a SQlite engine to read and work with SQlite database files.
//!
//! This is a work in progress and a learning project.
//!
//!
//! There are three components:
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
//!    let db = DB::new("sample.db")?;
//!    let mut executor = Executor::new(db);
//!    if let Some(response) = executor.execute(command)? {
//!        display_response(&response);
//!    }
//!```

pub mod db;
pub mod executor;
pub mod parser;
