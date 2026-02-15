//! Module that contains the sqlite leaf index structure
//!
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

    pub fn get_pointers<'a>(&self, where_clause: &Where, table: &'a Table) -> Result<Vec<usize>> {
        let mut retval = vec![];
        let cell_array = self.get_cell_pointer_array();
        let mut cursor = Cursor::new(cell_array);

        for _ in 0..self.cell_number {
            let offset = cursor.read_u16::<BigEndian>()? as usize;

            let mut record = Record::new(&self.get_slice(offset, None), table, false)?;

            let column = where_clause.get_identifier().unwrap();
            let field = record.take_field(column);
            if where_clause.evaluate(field.as_ref()) {
                if let RType::Num(rowid) = record.take_field("rowid").unwrap() {
                    retval.push(rowid as usize);
                }
            } else if retval.len() > 0 {
                break;
            }
        }
        Ok(retval)
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

    /// cell_pointer_array are pointers to page cells
    /// cells are records
    fn get_cell_pointer_array(&self) -> &[u8] {
        let buffer = self.get_page_buffer();
        let cell_number = self.cell_number;
        return &buffer[8..8 + cell_number as usize * 2];
    }

    /// Get a slice
    /// This function does not automaticaly shift the offset to after the file header
    /// in case of the page is the first page. This functions is used mostly to retrieve record
    fn get_slice(&self, start: usize, end: Option<usize>) -> &[u8] {
        if let Some(end_range) = end {
            &self.buffer[start..end_range]
        } else {
            &self.buffer[start..]
        }
    }
}
