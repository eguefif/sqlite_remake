use crate::db::Response;
use crate::db::parser::statement::Statement;

use anyhow::Result;

pub struct Executor {}

impl Executor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute_query(&self, query: &Statement) -> Result<Response> {
        let response = vec![];
        println!("{}", query);

        Ok(response)
    }
}
