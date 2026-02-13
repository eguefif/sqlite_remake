//! Module that contains the sqlite page structure
//!
//! A page can be of different types, see [PageType] enum.
//!
//! The first page is a special page as it contains the database header. It is stored
//! in the first 100 bytes of the file. The rest of the page is a normal page.
//! Note that for this page, the page header starts at 100 but the record offsets
//! are relative to the start of the page (0).
//!
//! For now, we only suport B-Tree pages
//! A page is composed of the following:
//! * a header [PageHeader]
//! * a cell pointer array: array of u16 offsets to the cells
//!
//! A `cell` contains a record. See [Record] module for more information about records.
//! But Cell format depends on the BTree type. See 1.6. B-tree Pages in
//! [Sqlite fileformat documentation](https://www.sqlite.org/fileformat.html)
use crate::{
    db::{fileformat::record::Record, table::Table},
    executor::db_response::RType,
    parser::where_clause::Where,
};
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

pub struct LeafIndex {
    buffer: Vec<u8>,
    pub page_number: usize,

    pub start_free: usize,
    pub cell_number: usize,
    pub start_content: usize,
    pub frag_number: u8,
}

impl LeafIndex {
    // Creates a new sqlite page
    // See documentation for the why https://www.sqlite.org/fileformat.html
    // THe first page contains the file header that measures 100 bytes.
    pub fn new(buffer: Vec<u8>, page_number: usize) -> Result<Self> {
        let mut cursor = if page_number == 1 {
            Cursor::new(&buffer[100..])
        } else {
            Cursor::new(&buffer[..])
        };

        let _ = cursor.read_u8()?;
        let start_free = cursor.read_u16::<BigEndian>()? as usize;
        let cell_number = cursor.read_u16::<BigEndian>()? as usize;
        let start_content = cursor.read_u16::<BigEndian>()? as usize;
        let frag_number = cursor.read_u8()?;

        Ok(Self {
            buffer,
            page_number,
            start_free,
            cell_number,
            start_content,
            frag_number,
        })
    }

    /// Get the database header
    /// This function only works for the first page
    pub fn get_db_header(&self) -> Option<&[u8]> {
        if self.page_number == 1 {
            Some(&self.buffer[0..100])
        } else {
            None
        }
    }

    /// Utility functions to automatically skip the first 100 bytes header
    /// if the page is the first page
    fn get_page_buffer(&self) -> &[u8] {
        // The first page contains the db metadata. It span from the byte 0
        // to the byte 100
        if self.page_number == 1 {
            &self.buffer[100..]
        } else {
            &self.buffer
        }
    }

    /// Get the number of records in the page
    /// A Cell contains a record, therefore, the number of record
    /// is the number of cells in the page
    pub fn get_record_number(&self) -> usize {
        self.cell_number
    }

    /// cell_pointer_array are pointers to page cells
    /// cells are records
    pub fn get_cell_pointer_array(&self) -> &[u8] {
        let buffer = self.get_page_buffer();
        let cell_number = self.cell_number;
        return &buffer[8..8 + cell_number as usize * 2];
    }

    /// Get a slice
    /// This function does not automaticaly shift the offset to after the file header
    /// in case of the page is the first page. This functions is used mostly to retrieve record
    pub fn get_slice(&self, start: usize, end: Option<usize>) -> &[u8] {
        if let Some(end_range) = end {
            &self.buffer[start..end_range]
        } else {
            &self.buffer[start..]
        }
    }

    pub fn get_all_records<'a>(&self, table: &'a Table) -> Result<Vec<Record<'a>>> {
        let mut rows = vec![];
        let cell_array = self.get_cell_pointer_array();
        let mut cursor = Cursor::new(cell_array);

        for _ in 0..self.get_record_number() {
            let offset = cursor.read_u16::<BigEndian>()? as usize;
            let record = Record::new(&self.get_slice(offset, None), table, false)?;
            rows.push(record);
        }
        Ok(rows)
    }

    /// This function is used to iterate over records in a page
    pub fn get_nth_record<'a>(&self, index: usize, schema_table: &'a Table) -> Result<Record<'a>> {
        let cell_array_offset = index * 2;
        let cell_array = self.get_cell_pointer_array();
        let mut cursor = Cursor::new(&cell_array[cell_array_offset..]);
        let offset = cursor.read_u16::<BigEndian>()? as usize;
        let record = Record::new(&self.get_slice(offset as usize, None), schema_table, false)
            .expect("Error: indexing record, file parsing failed");
        Ok(record)
    }

    pub fn get_pointers<'a>(&self, where_clause: &Where, table: &'a Table) -> Result<Vec<usize>> {
        let mut retval = vec![];
        let cell_array = self.get_cell_pointer_array();
        let mut cursor = Cursor::new(cell_array);

        for _ in 0..self.cell_number {
            let offset = cursor.read_u16::<BigEndian>()? as usize;

            let mut record = Record::new(&self.get_slice(offset, None), table, false)?;

            let column = where_clause.get_identifier().unwrap();
            let field = record.take_field(column);
            if where_clause.evaluate(field.as_ref()) == true {
                if let RType::Num(rowid) = record.take_field("rowid").unwrap() {
                    retval.push(rowid as usize);
                }
            } else if retval.len() > 0 {
                break;
            }
        }
        Ok(retval)
    }
}
