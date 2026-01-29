//! TODO: Add documentation
use crate::db::DB;
use crate::executor::db_response::Response;
use crate::parser::{Parser, select::SelectStatement, statement::Statement};
use anyhow::Result;

pub mod db_response;

pub struct Executor {
    db: DB,
}

impl Executor {
    pub fn new(db: DB) -> Self {
        Self { db }
    }

    /// Execute a command, which can be either a special command (like .dbinfo or .tables)
    /// or a SQL query.
    /// Returns None for special commands, or Some(Vec<(Query, Response)) for SQL queries.
    /// Response is a Vec<Vec<[Rtype](crate::executor::db_response)>>
    pub fn execute(&mut self, command: &str) -> Result<Option<Vec<(Statement, Response)>>> {
        match command {
            // TODO: change print, should return a response
            ".dbinfo" => self.db.metadata.print_metadata(),
            ".tables" => self.db.metadata.print_table_names(),
            _ => {
                return self.execute_queries(command);
            }
        }
        Ok(None)
    }

    fn execute_queries(&mut self, queries: &str) -> Result<Option<Vec<(Statement, Response)>>> {
        let parser = Parser::new(queries);
        let mut results: Vec<(Statement, Response)> = vec![];
        for query in parser {
            let statement = query?;
            let result = self.execute_query(&statement)?;
            if let Some(result) = result {
                results.push((statement, result));
            } else {
                return Ok(None);
            }
        }

        Ok(Some(results))
    }

    fn execute_query(&mut self, query: &Statement) -> Result<Option<Response>> {
        match query {
            Statement::Select(select_statement) => self.execute_select(select_statement),
        }
    }

    fn execute_select(&mut self, query: &SelectStatement) -> Result<Option<Response>> {
        let mut response = vec![]
        let Some(page) = self.db.get_table_page(&query.from_clause)? else {
            return Ok(None);
        };

        let rows = page.get_all_rows()?;
        for row in rows {
            // TODO: refactor record to contains directly the final type we have in RType
            // the FieldType should just be temporary. We can remove it. We need to store from
            // the beginning what we will provides.
        }
        Ok(Some(response))
    }
}
