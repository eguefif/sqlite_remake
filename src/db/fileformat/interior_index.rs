//! Module that contains the sqlite page structure
//!
//! A page can be of different types, see [PageType] enum.
//!
//! The first page is a special page as it contains the database header. It is stored
//! To write
//!
use anyhow::{Result, anyhow};
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

use crate::{
    db::{fileformat::record::Record, table::Table},
    parser::{token::Token, where_clause::Where},
};

pub struct InteriorIndex {
    buffer: Vec<u8>,
    pub page_number: usize,

    pub start_free: usize,
    pub cell_number: usize,
    pub start_content: usize,
    pub frag_number: u8,
    pub right_most_pointer: usize,
}

impl InteriorIndex {
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
        let right_most_pointer = cursor.read_u32::<BigEndian>()? as usize;

        Ok(Self {
            buffer,
            page_number,
            start_free,
            cell_number,
            start_content,
            frag_number,
            right_most_pointer,
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

    /// cell_pointer_array are pointers to page cells
    /// cells are records
    fn get_cell_pointer_array(&self) -> &[u8] {
        let buffer = self.get_page_buffer();
        let cell_number = self.cell_number;
        return &buffer[12..12 + cell_number as usize * 2];
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
        let mut records = vec![];
        let cells_buffer = self.get_cell_pointer_array();
        let mut cursor = Cursor::new(cells_buffer);
        for _ in 0..self.cell_number {
            let cell_pointer = cursor.read_u16::<BigEndian>()? as usize;
            let record = Record::new(&self.get_slice(cell_pointer + 4, None), table, false)?;
            records.push(record);
        }

        Ok(records)
    }

    pub fn get_all_pointers(&self) -> Result<Vec<usize>> {
        let mut index_pointers = vec![];
        let cells_buffer = self.get_cell_pointer_array();
        let mut cursor = Cursor::new(cells_buffer);
        for _ in 0..self.cell_number {
            let cell_pointer = cursor.read_u16::<BigEndian>()? as usize;
            let mut cell_cursor = Cursor::new(&self.buffer[cell_pointer..cell_pointer + 4]);
            let pointer = cell_cursor.read_u32::<BigEndian>()?;
            index_pointers.push(pointer as usize);
        }
        index_pointers.push(self.right_most_pointer);

        Ok(index_pointers)
    }

    pub fn get_record_number(&self) -> usize {
        self.cell_number
    }

    /// This function is used to iterate over records in a page
    pub fn get_nth_record<'a>(
        &self,
        index: usize,
        schema_table: &'a Table,
    ) -> Result<(usize, Record<'a>)> {
        if index > self.get_record_number() {
            return Err(anyhow!(""));
        }
        let cell_array_offset = index * 2;
        let cell_array = self.get_cell_pointer_array();
        let mut cursor = Cursor::new(&cell_array[cell_array_offset..]);
        let offset = cursor.read_u16::<BigEndian>()? as usize;
        let mut buf_cursor = Cursor::new(&self.buffer[offset as usize..]);
        let pointer_page = buf_cursor.read_u32::<BigEndian>()? as usize;
        let record = Record::new(
            &self.get_slice(offset + 4 as usize, None),
            schema_table,
            false,
        )
        .expect("Error: indexing record, file parsing failed");
        Ok((pointer_page, record))
    }

    pub fn is_where<'a>(&self, where_clause: &Where, table: &'a Table) -> Result<Option<usize>> {
        let cell_array = self.get_cell_pointer_array();
        let mut cursor = Cursor::new(cell_array);

        for i in 0..self.get_record_number() {
            let offset = cursor.read_u16::<BigEndian>()? as usize;
            let mut record = Record::new(&self.get_slice(offset + 4, None), table, false)?;
            let column = where_clause.get_identifier().unwrap();
            let field = record.take_field(column);
            if where_clause.evaluate(field.as_ref()) == true {
                return Ok(Some(i));
            }
        }
        Ok(None)
    }

    pub fn get_next_page<'a>(&self, where_clause: &Where, table: &'a Table) -> Result<usize> {
        let cell_array = self.get_cell_pointer_array();
        let mut cursor = Cursor::new(cell_array);

        let less_than_where = Where::from_where(where_clause, Token::LTEQ);
        for _ in 0..self.get_record_number() {
            let offset = cursor.read_u16::<BigEndian>()? as usize;
            let mut buf_cursor = Cursor::new(&self.buffer[offset as usize..]);
            let pointer_page = buf_cursor.read_u32::<BigEndian>()? as usize;

            let mut record = Record::new(&self.get_slice(offset + 4, None), table, false)?;

            let column = where_clause.get_identifier().unwrap();
            let field = record.take_field(column);
            if less_than_where.evaluate(field.as_ref()) == true {
                return Ok(pointer_page);
            }
        }
        Ok(self.right_most_pointer)
    }
}
