//! Api to execute a raw sql string or a Sqlite special command
use crate::db::DB;
use crate::db::fileformat::record::Record;
use crate::db::table::Table;
use crate::executor::db_response::{RType, Response};
use crate::parser::function::FuncCall;
use crate::parser::identifier::{Identifier, VType};
use crate::parser::select::{SelectClause, SelectItem};
use crate::parser::token::Command;
use crate::parser::where_clause::Where;
use crate::parser::{Parser, select::SelectStatement, statement::Statement};
use anyhow::{Result, anyhow};

pub mod db_response;

enum Plan<'a> {
    IndexScan(&'a Where),
    SeqScan,
}

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
    pub fn execute(&mut self, command: &str) -> Result<Vec<(Statement, Response)>> {
        let parser = Parser::new(command);
        let mut results: Vec<(Statement, Response)> = vec![];
        for query in parser {
            let statement = query?;
            let result = self.execute_query(&statement)?;
            if let Some(result) = result {
                results.push((statement, result));
            }
        }

        Ok(results)
    }

    fn execute_query(&mut self, query: &Statement) -> Result<Option<Response>> {
        match query {
            Statement::Select(select_statement) => self.execute_select_statement(select_statement),
            Statement::Command(Command::DBinfo) => self.db.metadata.get_metadata(),
            Statement::Command(Command::Tables) => self.db.metadata.get_table_names(),
        }
    }

    // NOTE: about lifetime here.
    // Table has to live longer than each indivual records. Each records contains a
    // HashMap of columns. Keys are &str to the table definition column's name.
    // Records and Table lives only in this function scope.
    fn execute_select_statement(&mut self, query: &SelectStatement) -> Result<Option<Response>> {
        let Some(table) = self.db.take_table(&query.from_clause) else {
            println!("table: {:?}", query);
            return Ok(None);
        };

        // Check for index and use it
        let plan = self.plan_execution(query, &table);
        let records = match plan {
            Plan::IndexScan(where_clause) => self.index_scan(where_clause, &table)?,
            Plan::SeqScan => self.seq_scan(&query, &table)?,
        };

        let response = records
            .into_iter()
            .map(|record| apply_select_clause(record, &query.select_clause, &table))
            .collect::<Result<Response>>()?;

        if let Some(func) = query.select_clause.get_function() {
            Ok(Some(vec![execute_function(&response, func)]))
        } else {
            Ok(Some(response))
        }
    }

    fn plan_execution<'a>(&self, query: &'a SelectStatement, table: &Table) -> Plan<'a> {
        if let Some(where_clause) = &query.where_clause {
            if table.has_index_on(where_clause) {
                return Plan::IndexScan(where_clause);
            }
        }
        Plan::SeqScan
    }

    fn index_scan<'a>(
        &mut self,
        where_clause: &Where,
        table: &'a Table,
    ) -> Result<Vec<Record<'a>>> {
        self.db.index_scan(&table, where_clause)
    }

    fn seq_scan<'a>(
        &mut self,
        query: &SelectStatement,
        table: &'a Table,
    ) -> Result<Vec<Record<'a>>> {
        let records = self.db.seq_scan(&query.where_clause, &table)?;
        Ok(records)
    }
}

fn execute_function(records: &Response, func: &FuncCall) -> Vec<RType> {
    match func.function_name.as_str() {
        "count" => vec![RType::Num(records.len() as i64)],
        _ => vec![],
    }
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
