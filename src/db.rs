//! A simple database engine that can read a database file, parse queries, and return results. It contains struct to represent the database, its metadata, and responses.
//!
//! This module has two submodules:
//! * [fileformat] contains what we need to parser the sqlite file
//! * [dbmetadata] contains all the information on the sqlite database
//!
use crate::db::dbmetadata::DBMetadata;
use crate::db::fileformat::page::Page;
use crate::db::table::Table;
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

    pub fn get_page(&mut self, root_page: usize) -> Result<Page> {
        let mut page_buffer = self.get_new_page_buffer();
        // Page are numbered from 1, we need to subtract 1 to get the offset
        let offset = ((root_page - 1) * self.page_size) as u64;
        self.buf_reader.seek(SeekFrom::Start(offset))?;
        self.buf_reader.read_exact(&mut page_buffer)?;
        Page::new(page_buffer, root_page)
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
