use crate::db::DB;
use crate::db::db_response::Response;
use crate::db::parser::Parser;
use crate::db::parser::statement::Statement;
use anyhow::Result;

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
    /// A Response is a vector of rows, where each row is a vector of [RType values][RType].
    pub fn execute(&mut self, command: &str) -> Result<Option<Vec<(Statement, Response)>>> {
        match command {
            ".dbinfo" => self.db.metadata.print_metadata(),
            ".tables" => self.db.metadata.print_table_names(),
            _ => {
                return self.execute_queries(command);
            }
        }
        Ok(None)
    }

    fn execute_queries(&self, queries: &str) -> Result<Option<Vec<(Statement, Response)>>> {
        let parser = Parser::new(queries);
        let mut results: Vec<(Statement, Response)> = vec![];
        for query in parser {
            let statement = query?;
            let result = self.execute_query(&statement)?;
            results.push((statement, result));
        }

        Ok(Some(results))
    }

    fn execute_query(&self, query: &Statement) -> Result<Response> {
        let response = vec![];

        Ok(response)
    }
}
