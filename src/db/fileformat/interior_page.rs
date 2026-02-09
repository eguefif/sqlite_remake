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
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

pub struct InteriorPage {
    buffer: Vec<u8>,
    pub page_number: usize,

    pub btree_type: BTreeType,
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

        let btree_type = BTreeType::new(cursor.read_u8()?);
        let start_free = cursor.read_u16::<BigEndian>()? as usize;
        let cell_number = cursor.read_u16::<BigEndian>()? as usize;
        let start_content = cursor.read_u16::<BigEndian>()? as usize;
        let frag_number = cursor.read_u8()?;
        let right_most_pointer = cursor.read_u32::<BigEndian>()? as usize;

        Ok(Self {
            buffer,
            page_number,
            btree_type,
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

    // TODO: This part has a bug. We got inconsisten page value
    // when comparing with hexdump. There are 133 page in the file and
    // we get
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
}

#[derive(PartialEq, Debug)]
pub enum PageType {
    BTree(BTreeType),
    FreeList,
    Overflow,
    PointerMap,
    LockByte,
}

#[derive(PartialEq, Debug)]
pub enum BTreeType {
    InteriorIndex,
    InteriorPage,
    LeafIndex,
    LeafPage,
}

impl BTreeType {
    pub fn new(number_type: u8) -> Self {
        match number_type {
            0x02 => BTreeType::InteriorIndex,
            0x05 => BTreeType::InteriorPage,
            0x0a => BTreeType::LeafIndex,
            0x0d => BTreeType::LeafPage,
            _ => panic!("Error: Number type invalid"),
        }
    }
}
