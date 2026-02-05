//! Statement module to handle statement
//! A statement can be of several type:
//! * select
//! * update
//! * insert
//!
//! For each statement, there are the following clauses:
//! * Select => mandatory, not to confuse with the statement type
//! * From
//! * Where
//!
use crate::parser::{select::SelectStatement, token::Command};
use std::fmt;

#[derive(Debug)]
pub enum Statement {
    Select(SelectStatement),
    Command(Command),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Statement::Select(statement) => write!(f, "{}", statement),
            Statement::Command(cmd) => write!(f, "{}", cmd),
        }
    }
}
