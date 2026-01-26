//! A simple database engine that can read a database file, parse queries, and return results.
//! It s
use crate::db::db_response::{RType, Response};
use crate::db::dbmetadata::DBMetadata;
use crate::fileformat::page::Page;
use crate::parser::{Parser, query::Query};
use anyhow::{Result, anyhow};
use std::fs::File;
use std::io::BufReader;
use std::io::{Read, Seek, SeekFrom};

pub mod db_response;
pub mod dbmetadata;
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
        let page_size = DB::get_page_size(&mut buf_reader)? as usize;

        let mut buffer = Vec::new();
        buffer.resize(page_size as usize, 0);
        buf_reader.read_exact(&mut buffer)?;
        buf_reader.rewind()?;
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
        let offset = ((root_page - 1) * self.page_size) as u64;
        self.buf_reader.seek(SeekFrom::Start(offset))?;
        self.buf_reader.read_exact(&mut buffer)?;
        Page::new(buffer, root_page)
    }

    pub fn execute(&mut self, command: &str) -> Result<Option<Vec<Response>>> {
        match command {
            ".dbinfo" => self.metadata.print_metadata(),
            ".tables" => self.metadata.print_table_names(),
            _ => {
                return self.process_query(command.to_string());
            }
        }
        Ok(None)
    }

    pub fn process_query(&mut self, query_str: String) -> Result<Option<Vec<Response>>> {
        let mut parser = Parser::new(&query_str);
        let mut responses: Vec<Response> = vec![];
        parser.parse()?;
        for query in parser.queries {
            let response = self.execute_query(&query)?;
            responses.push(response)
        }
        Ok(Some(responses))
    }

    fn execute_query(&mut self, query: &Query) -> Result<Response> {
        let Some(table) = self.metadata.schema.get(&query.from) else {
            return Err(anyhow!("The table does not exists"));
        };

        let rootpage = table.rootpage;
        let mut response: Response = vec![];

        let mut cols: Vec<&str> = vec![];
        for value in query.select.iter() {
            cols.push(value);
        }

        let mut col_indexes: Vec<usize> = vec![];
        for col_name in cols {
            if col_name.to_lowercase().contains("count") {
                let page = self.get_page(rootpage)?;
                let row_count = page.get_record_number() as i64;
                let cell = RType::Num(row_count);
                return Ok(vec![vec![cell]]);
            } else {
                col_indexes.push(table.get_col_index(col_name));
            }
        }

        let page = self.get_page(rootpage)?;
        for n in 0..page.get_record_number() {
            let mut row = vec![];
            let record = page.get_nth_record(n);
            for col_index in col_indexes.iter() {
                row.push(record.get_col(*col_index))
            }
            response.push(row);
        }

        Ok(response)
    }
}
