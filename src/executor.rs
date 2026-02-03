//! TODO: Add documentation
use crate::db::DB;
use crate::db::fileformat::page::Page;
use crate::db::fileformat::record::Record;
use crate::db::table::Table;
use crate::executor::db_response::{RType, Response};
use crate::parser::function::FuncCall;
use crate::parser::identifier::{Identifier, VType};
use crate::parser::select::{SelectClause, SelectItem};
use crate::parser::where_clause::Where;
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
        let Some(table) = self.db.take_table(&query.from_clause) else {
            return Ok(None);
        };

        // TODO: we need to replace the ID by rowid when constructing record
        let page = self.db.get_page(table.get_root_page())?;
        let records = page.get_all_records(&table)?;
        let response = records
            .into_iter()
            .filter(|record| {
                if let Some(where_clause) = &query.where_clause {
                    apply_where_clause(record, &where_clause)
                } else {
                    true
                }
            })
            .map(|record| apply_select_clause(record, &query.select_clause, &table))
            .collect();

        if let Some(func) = query.select_clause.get_function() {
            Ok(Some(vec![execute_function(&page, func)]))
        } else {
            Ok(Some(response))
        }
    }
}

fn execute_function(page: &Page, func: &FuncCall) -> Vec<RType> {
    match func.function_name.as_str() {
        "count" => vec![RType::Num(page.get_record_number() as i64)],
        _ => vec![],
    }
}

fn apply_where_clause(record: &Record, where_clause: &Where) -> bool {
    // For now, we assume there is only one identifier in the where clause
    if let Some(identifier) = where_clause.get_identifier() {
        if identifier != "id" {
            let identifier_value = record.get_column_value(identifier);
            return where_clause.evaluate(Some(identifier_value));
        }
        return where_clause.evaluate(Some(&RType::Num(record.rowid as i64)));
    };
    where_clause.evaluate(None)
}

fn apply_select_clause(mut record: Record, select: &SelectClause, table: &Table) -> Vec<RType> {
    let record_id = RType::Num(record.rowid as i64);
    let mut selected_row = vec![];
    let col_names = get_selected_colname(select, table);

    // TODO: when building record, should replace id with rowid
    for col_name in col_names {
        if col_name == "id" {
            selected_row.push(record_id.clone());
        } else {
            selected_row.push(record.take_field(col_name).unwrap())
        }
    }
    selected_row
}

fn get_selected_colname<'a>(select_clause: &'a SelectClause, table: &'a Table) -> Vec<&'a str> {
    let mut col_indexes = vec![];
    if select_clause.items.len() == 0 {
        return col_indexes;
    }

    for item in select_clause.items.iter() {
        match item {
            SelectItem::Identifier(Identifier {
                value: VType::Str(col_name),
            }) => {
                col_indexes.push(col_name);
            }
            SelectItem::Star => {
                return table
                    .cols_name
                    .iter()
                    .map(|str| str.as_ref())
                    .collect::<Vec<&'a str>>();
            }
            SelectItem::Function(_) => {}
            _ => panic!("Should always be a valid select item: {:?}", item),
        }
    }

    col_indexes
}
