//! A simple database engine that can read a database file, parse queries, and return results. It contains struct to represent the database, its metadata, and responses.
//!
//! # Example
//! ```
//! use crate::db::DB;
//! let mut db = DB::new("test.db").unwrap();
//! let response = db.execute("SELECT name FROM users;").unwrap();
//! ```
//!
use crate::db::db_response::{RType, Response};
use crate::db::dbmetadata::DBMetadata;
use crate::db::fileformat::page::Page;
use crate::db::fileformat::record::Record;
use crate::db::parser::{
    Parser,
    query::{Query, SelectType, Statement},
};
use anyhow::{Result, anyhow};
use std::fs::File;
use std::io::BufReader;
use std::io::{Read, Seek, SeekFrom};

pub mod db_response;
pub mod dbmetadata;
pub mod fileformat;
pub mod parser;
pub mod table;

pub struct DB {
    metadata: DBMetadata,
    page_size: usize,
    buf_reader: BufReader<File>,
}

impl DB {
    pub fn new(filename: &str) -> Result<Self> {
        let file = File::open(filename)?;
        let mut buf_reader = BufReader::new(file);
        // We need page_size to read pages. The page_size is defined in the database header.
        let page_size = DB::get_page_size(&mut buf_reader)? as usize;

        // We read the first page to build the metadata
        let mut buffer = Vec::new();
        buffer.resize(page_size as usize, 0);
        buf_reader.read_exact(&mut buffer)?;
        let page = Page::new(buffer, 1)?;
        let metadata = DBMetadata::new(page);

        Ok(Self {
            metadata,
            page_size,
            buf_reader,
        })
    }

    fn get_page_size(buf_reader: &mut BufReader<File>) -> Result<u16> {
        let mut header: [u8; 100] = [0; 100];
        buf_reader.read_exact(&mut header)?;
        buf_reader.rewind()?;
        Ok(u16::from_be_bytes([header[16], header[17]]))
    }

    fn get_page(&mut self, root_page: usize) -> Result<Page> {
        let mut buffer = Vec::new();
        buffer.resize(self.page_size, 0);
        // Page are numbered from 1, we need to subtract 1 to get the offset
        let offset = ((root_page - 1) * self.page_size) as u64;
        self.buf_reader.seek(SeekFrom::Start(offset))?;
        self.buf_reader.read_exact(&mut buffer)?;
        Page::new(buffer, root_page)
    }

    /// Execute a command, which can be either a special command (like .dbinfo or .tables)
    /// or a SQL query.
    /// Returns None for special commands, or Some(Vec<(Query, Response)) for SQL queries.
    /// A Response is a vector of rows, where each row is a vector of [RType values][RType].
    pub fn execute(&mut self, command: &str) -> Result<Option<Vec<(Query, Response)>>> {
        match command {
            ".dbinfo" => self.metadata.print_metadata(),
            ".tables" => self.metadata.print_table_names(),
            _ => {
                return self.process_queries(command.to_string());
            }
        }
        Ok(None)
    }

    fn process_queries(&mut self, query_str: String) -> Result<Option<Vec<(Query, Response)>>> {
        let parser = Parser::new(&query_str);
        let mut responses: Vec<(Query, Response)> = vec![];
        for query in parser.into_iter() {
            let query = query?;
            let response = self.execute_query(&query)?;
            responses.push((query, response))
        }
        Ok(Some(responses))
    }

    fn execute_query(&mut self, query: &Query) -> Result<Response> {
        let Some(table) = self.metadata.schema.get(&query.from) else {
            return Err(anyhow!("The table does not exists"));
        };

        let rootpage = table.rootpage;
        let mut response: Response = vec![];

        // We build a vec of indexes of columnss that select needs
        let col_indexes: Vec<usize> = self.get_column_indexes(&query.select, &table)?;

        let where_statement = &query.wh.get(0);
        let where_col_index = if let Some(where_statement) = where_statement {
            Some(table.get_col_index(&where_statement.left))
        } else {
            None
        };

        // We retrieve the cols values for each row
        let page = self.get_page(rootpage)?;
        if col_indexes.len() == 0 {
            return Ok(vec![vec![RType::Num(page.get_record_number() as i64)]]);
        }
        for n in 0..page.get_record_number() {
            let mut row = vec![];
            let record = page.get_nth_record(n);
            if let (Some(where_col_index), Some(where_statement)) =
                (where_col_index, where_statement)
            {
                if !self.match_where(&record, where_col_index, &where_statement) {
                    continue;
                }
            }
            for col_index in col_indexes.iter() {
                row.push(record.get_col(*col_index));
            }
            response.push(row);
        }

        Ok(response)
    }

    fn match_where(
        &self,
        record: &Record,
        where_col_index: usize,
        where_statement: &Statement,
    ) -> bool {
        let value = record.get_col(where_col_index);
        where_statement.cmp(value)
    }

    fn get_column_indexes(
        &self,
        select: &Vec<SelectType>,
        table: &table::Table,
    ) -> Result<Vec<usize>> {
        let mut col_indexes = vec![];
        if let SelectType::Function((ref func, _)) = select[0] {
            if func == "count" {
                return Ok(col_indexes);
            } else {
                return Err(anyhow!("Unsupported function in select"));
            }
        } else {
            for select_type in select.iter() {
                if let SelectType::Value(value) = select_type {
                    let col_index = table.get_col_index(value);
                    col_indexes.push(col_index);
                } else {
                    return Err(anyhow!("Unsupported select type"));
                }
            }
            Ok(col_indexes)
        }
    }
}
