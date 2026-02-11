//! A simple database engine that can read a database file, parse queries, and return results. It contains struct to represent the database, its metadata, and responses.
//!
//! This module has two submodules:
//! * [fileformat] contains what we need to parser the sqlite file
//! * [dbmetadata] contains all the information on the sqlite database
//!
use crate::db::dbmetadata::DBMetadata;
use crate::db::fileformat::PageType;
use crate::db::fileformat::interior_index::InteriorIndex;
use crate::db::fileformat::interior_page::InteriorPage;
use crate::db::fileformat::page::Page;
use crate::db::fileformat::record::Record;
use crate::db::table::Table;
use crate::executor::db_response::RType;
use crate::parser::where_clause::Where;
use anyhow::Result;
use std::fs::File;
use std::io::BufReader;
use std::io::{Read, Seek, SeekFrom};

pub mod dbmetadata;
pub mod fileformat;
pub mod table;

pub struct DB {
    pub metadata: DBMetadata,
    pub page_size: usize,
    pub buf_reader: BufReader<File>,
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
        let metadata = DBMetadata::new(page)?;

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

    pub fn take_table(&mut self, tablename: &str) -> Option<Table> {
        self.metadata.take_table(tablename)
    }

    pub fn index_scan<'a>(
        &mut self,
        table: &'a Table,
        where_clause: &Where,
    ) -> Result<Vec<Record<'a>>> {
        // Retrieve all indexes pointers
        let index = table.indexes.first().unwrap();
        let root_page = index.get_root_page();
        let PageType::InteriorIndex(page) = self.get_page(root_page)? else {
            panic!();
        };
        let mut retval = vec![];
        let mut index_values = self.traverse_btree_index(page, index)?;

        // For each pointers, check if they match the where clause
        for index in index_values.iter_mut() {
            let fields = index.take_fields();
            let column_name = where_clause.get_identifier().unwrap();
            let value = fields.get(column_name);
            if where_clause.evaluate(value) {
                if let RType::Num(rowid) = fields.get("rowid").unwrap() {
                    let root_page = table.get_root_page();
                    if let Some(value) = self.get_record(root_page, *rowid as usize, table) {
                        retval.push(value);
                    }
                }
            }
        }
        // 1. Get index and apply where => return a list of rowid
        // 2. Retrieve each row using self.get_rowid(rowid)

        Ok(retval)
    }

    fn traverse_btree_index<'a>(
        &mut self,
        page: InteriorIndex,
        index: &'a Table,
    ) -> Result<Vec<Record<'a>>> {
        let mut retval = vec![];
        let pointers = page.get_all_pointers()?;
        for pointer in pointers {
            let page = self.get_page(pointer)?;
            match page {
                PageType::InteriorIndex(page) => {
                    let mut page_records = page.get_all_records(index)?;
                    let mut records = self.traverse_btree_index(page, index)?;
                    retval.append(&mut page_records);
                    retval.append(&mut records)
                }
                PageType::LeafIndex(page) => {
                    let mut records = page.get_all_records(index)?;
                    retval.append(&mut records);
                }
                _ => panic!(),
            }
        }
        Ok(retval)
    }

    fn get_record<'a>(
        &mut self,
        page_number: usize,
        rowid: usize,
        table: &'a Table,
    ) -> Option<Record<'a>> {
        let page = self.get_page(page_number).unwrap();
        match page {
            PageType::Page(page) => {
                for i in 0..page.get_record_number() {
                    let record = page.get_nth_record(i, table).unwrap();
                    if record.rowid == rowid {
                        return Some(record);
                    }
                }
                None
            }
            PageType::InteriorPage(page) => {
                let pointers = page.get_all_pointers().unwrap();
                for pointer in pointers {
                    match self.get_record(pointer, rowid, table) {
                        Some(record) => return Some(record),
                        None => continue,
                    }
                }
                None
            }
            _ => panic!(),
        }
    }

    pub fn seq_scan<'a>(&mut self, table: &'a Table) -> Result<Vec<Record<'a>>> {
        let page = self.get_page(table.get_root_page())?;

        match page {
            PageType::InteriorPage(page) => self.traverse_btree(page, table),
            PageType::Page(page) => page.get_all_records(table),
            _ => panic!(),
        }
    }

    fn traverse_btree<'a>(
        &mut self,
        page: InteriorPage,
        table: &'a Table,
    ) -> Result<Vec<Record<'a>>> {
        let mut retval = vec![];
        let pointers = page.get_all_pointers()?;
        for pointer in pointers {
            let page = self.get_page(pointer)?;
            match page {
                PageType::Page(page) => {
                    let mut records = page.get_all_records(table)?;
                    retval.append(&mut records)
                }
                PageType::InteriorPage(page) => {
                    let mut records = self.traverse_btree(page, table)?;
                    retval.append(&mut records)
                }
                _ => panic!(),
            }
        }
        Ok(retval)
    }

    pub fn get_page(&mut self, page_number: usize) -> Result<PageType> {
        let mut page_buffer = self.get_new_page_buffer();
        // Page are numbered from 1, we need to subtract 1 to get the offset
        let offset = ((page_number - 1) * self.page_size) as u64;
        self.buf_reader.seek(SeekFrom::Start(offset))?;

        self.buf_reader.read_exact(&mut page_buffer)?;
        let page = PageType::from_buffer(page_buffer, page_number);
        page
    }

    // Utility function that is used to provide a buffer
    // that can be used with read_exact or read.
    // Using with_capacity does not work.
    fn get_new_page_buffer(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.resize(self.page_size, 0);
        buffer
    }
}
