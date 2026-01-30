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
use crate::db::fileformat::record::Record;
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

// TODO: handle other page types using a trait
// The trait should provides methods to :
//   - get the number of records
//   - get a page
//  We will need a page factory that take a buffer and a page_number

pub struct Page {
    pub buffer: Vec<u8>,
    pub page_number: usize,
    pub page_header: PageHeader,
}

impl Page {
    // Creates a new sqlite page
    // See documentation for the why https://www.sqlite.org/fileformat.html
    // THe first page contains the file header that measures 100 bytes.
    pub fn new(buffer: Vec<u8>, page_number: usize) -> Result<Self> {
        let page_header;
        if page_number == 1 {
            page_header = PageHeader::new(&buffer[100..])?;
        } else {
            page_header = PageHeader::new(&buffer)?;
        }
        Ok(Self {
            buffer,
            page_number,
            page_header,
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
        self.page_header.cell_number
    }

    /// cell_pointer_array are pointers to page cells
    /// cells are records
    pub fn get_cell_pointer_array(&self) -> &[u8] {
        let buffer = self.get_page_buffer();
        let cell_number = self.page_header.cell_number;
        if self.page_header.btree_type == BTreeType::InteriorPage {
            return &buffer[12..12 + cell_number as usize * 2];
        } else {
            return &buffer[8..8 + cell_number as usize * 2];
        }
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

    pub fn get_all_records(&self) -> Result<Vec<Record>> {
        let mut rows = vec![];
        let cell_array = self.get_cell_pointer_array();
        let mut cursor = Cursor::new(cell_array);

        for _ in 0..self.get_record_number() {
            let offset = cursor.read_u16::<BigEndian>()? as usize;
            let record = Record::new(&self.get_slice(offset, None))?;
            rows.push(record);
        }

        Ok(rows)
    }

    /// This function is used to iterate over records in a page
    pub fn get_nth_record(&self, index: usize) -> Record {
        let cell_array_offset = index * 2;
        let cell_array = self.get_cell_pointer_array();
        let mut cursor = Cursor::new(&cell_array[cell_array_offset..]);
        let offset = cursor.read_u16::<BigEndian>().unwrap() as usize;
        let record = Record::new(&self.get_slice(offset as usize, None))
            .expect("Error: indexing record, file parsing failed");
        record
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

pub struct PageHeader {
    pub btree_type: BTreeType,
    pub start_free: usize,
    pub cell_number: usize,
    pub start_content: usize,
    pub frag_number: u8,
    pub right_most_pointer: usize,
}

impl PageHeader {
    fn new(buffer: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(buffer);
        Ok(PageHeader {
            btree_type: BTreeType::new(cursor.read_u8()?),
            start_free: cursor.read_u16::<BigEndian>()? as usize,
            cell_number: cursor.read_u16::<BigEndian>()? as usize,
            start_content: cursor.read_u16::<BigEndian>()? as usize,
            frag_number: cursor.read_u8()?,
            right_most_pointer: cursor.read_u16::<BigEndian>()? as usize,
        })
    }
}
