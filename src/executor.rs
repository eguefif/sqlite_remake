//! TODO: Add documentation
use crate::db::DB;
use crate::db::table::Table;
use crate::executor::db_response::{RType, Response};
use crate::parser::identifier::{Identifier, VType};
use crate::parser::select::{SelectClause, SelectItem};
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
        let Some(table) = self.db.take_table(&query.from_clause) else {
            return Ok(None);
        };

        let page = self.db.get_page(table.get_root_page())?;
        let records = page.get_all_records()?;
        for mut record in records {
            let record_id = RType::Num(record.rowid as i64);
            let row = record.take_fields();
            let row = apply_select_clause(record_id, row, &query.select_clause, &table);

            response.push(row);
        }

        Ok(Some(response))
    }
}

fn apply_select_clause(
    record_id: RType,
    row: Vec<RType>,
    select: &SelectClause,
    table: &Table,
) -> Vec<RType> {
    let mut selected_row = vec![];
    let Some(cols_index_to_take) = get_col_indexes_to_take(select, table) else {
        return selected_row;
    };
    if cols_index_to_take.len() == 0 {
        for entry in row.into_iter() {
            selected_row.push(entry);
        }
        selected_row[0] = record_id;
        return selected_row;
    }

    if cols_index_to_take.contains(&0) {
        selected_row.push(record_id);
    }

    for (i, entry) in row.into_iter().enumerate() {
        if cols_index_to_take.contains(&i) {
            if i == 0 {
                continue;
            } else {
                selected_row.push(entry);
            }
        }
    }
    selected_row
}

fn get_col_indexes_to_take(select_clause: &SelectClause, table: &Table) -> Option<Vec<usize>> {
    let mut col_indexes = vec![];
    if select_clause.items.len() == 0 {
        return None;
    }

    for item in select_clause.items.iter() {
        match item {
            SelectItem::Identifier(Identifier {
                value: VType::Str(col_name),
            }) => {
                col_indexes.push(table.get_col_index(col_name));
            }
            SelectItem::Star => return Some(vec![]),
            _ => panic!(),
        }
    }

    Some(col_indexes)
}
