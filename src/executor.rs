//! TODO: Add documentation
use crate::db::DB;
use crate::executor::db_response::{RType, Response};
use crate::parser::identifier::{Identifier, VType};
use crate::parser::select::SelectItem;
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
            Statement::Select(select_statement) => self.execute_select_statement(select_statement),
        }
    }

    fn execute_select_statement(&mut self, query: &SelectStatement) -> Result<Option<Response>> {
        let mut response = vec![];
        let Some(page) = self.db.get_table_page(&query.from_clause)? else {
            return Ok(None);
        };

        let records = page.get_all_records()?;
        for mut record in records {
            let record_id = RType::Num(record.rowid as i64);
            let _cols_index_to_take =
                self.get_col_indexes_to_take(&query.from_clause, &query.select_clause);
            let mut row = record.take_fields();
            row[0] = record_id;

            response.push(row);
        }

        Ok(Some(response))
    }

    fn get_col_indexes_to_take(
        &self,
        from_clause: &str,
        select_clause: &crate::parser::select::SelectClause,
    ) -> Vec<usize> {
        let mut col_indexes = vec![];
        let Some(table) = self.db.metadata.schema.get(from_clause) else {
            panic!("Executor: parsing, table should exist at this point")
        };

        for item in select_clause.items.iter() {
            if let SelectItem::Identifier(Identifier {
                value: VType::Str(col_name),
            }) = item
            {
                col_indexes.push(table.get_col_index(col_name));
            }
        }

        col_indexes
    }
}
