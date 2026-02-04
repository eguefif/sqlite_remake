//! Api to execute a raw sql string or a Sqlite special command
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
use anyhow::{Result, anyhow};

pub mod db_response;

pub struct Executor {
    db: DB,
}

impl Executor {
    pub fn new(db: DB) -> Self {
        Self { db }
    }

    /// Execute a command.
    /// There are two types of command:
    /// * special commands: .dbinfo, .tables
    /// * a SQL query.
    /// Returns None for special commands, or Some(Vec<(Query, Response)) for SQL queries.
    /// Response is a Vec<Vec<[Rtype](crate::executor::db_response)>>
    pub fn execute(&mut self, command: &str) -> Result<Option<Vec<(Statement, Response)>>> {
        match command {
            ".dbinfo" => Ok(Some(vec![(
                Statement::Command(command.to_string()),
                self.db.metadata.get_metadata(),
            )])),
            ".tables" => Ok(Some(vec![(
                Statement::Command(command.to_string()),
                self.db.metadata.get_table_names(),
            )])),
            _ => self.execute_queries(command),
        }
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
            _ => {
                todo!(
                    "Refactor this part: it is because of the return type of execute which needs Statement. I added a Statement Command for .dbinfo"
                )
            }
        }
    }

    fn execute_select_statement(&mut self, query: &SelectStatement) -> Result<Option<Response>> {
        let Some(table) = self.db.take_table(&query.from_clause) else {
            return Ok(None);
        };

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
            .collect::<Result<Response>>()?;

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
        let identifier_value = record.get_column_value(identifier);
        return where_clause.evaluate(Some(identifier_value));
    };
    where_clause.evaluate(None)
}

fn apply_select_clause(
    mut record: Record,
    select: &SelectClause,
    table: &Table,
) -> Result<Vec<RType>> {
    let mut selected_row = vec![];
    let col_names = get_selected_colname(select, table);

    for col_name in col_names {
        if let Some(field) = record.take_field(col_name) {
            selected_row.push(field)
        } else {
            return Err(anyhow!("Select clause: invalid columna name: {}", col_name));
        }
    }
    Ok(selected_row)
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
