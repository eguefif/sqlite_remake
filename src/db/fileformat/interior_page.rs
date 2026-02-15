//! Module that contains the sqlite interior page structure
//!
//! [Sqlite fileformat documentation](https://www.sqlite.org/fileformat.html)
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

use crate::db::fileformat::types::Varint;

pub struct InteriorPage {
    buffer: Vec<u8>,
    pub page_number: usize,

    pub start_free: usize,
    pub cell_number: usize,
    pub start_content: usize,
    pub frag_number: u8,
    pub right_most_pointer: usize,
}

impl InteriorPage {
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

    pub fn get_next_page_number(&self, rowid: usize) -> Result<usize> {
        let cells_buffer = self.get_cell_pointer_array();
        let mut cursor = Cursor::new(cells_buffer);
        for _ in 0..self.cell_number {
            let cell_pointer = cursor.read_u16::<BigEndian>()? as usize;
            let mut cell_cursor = Cursor::new(&self.buffer[cell_pointer..]);
            let page_pointer = cell_cursor.read_u32::<BigEndian>()?;
            let cell_rowid = Varint::from_cursor(&mut cell_cursor)?;
            if rowid <= cell_rowid.varint as usize {
                return Ok(page_pointer as usize);
            }
        }
        Ok(self.right_most_pointer as usize)
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
}
