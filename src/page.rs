use crate::record::Record;
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

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

    // This function only works for the first page
    pub fn get_db_header(&self) -> Option<&[u8]> {
        if self.page_number == 1 {
            Some(&self.buffer[0..100])
        } else {
            None
        }
    }

    fn get_page_buffer(&self) -> &[u8] {
        // The first page contains the db metadata. It span from the byte 0
        // to the byte 100
        if self.page_number == 1 {
            &self.buffer[100..]
        } else {
            &self.buffer
        }
    }

    pub fn get_record_number(&self) -> usize {
        self.page_header.cell_number
    }

    // cell_pointer_array are pointers to page cells
    // cells are records
    pub fn get_cell_pointer_array(&self) -> &[u8] {
        let buffer = self.get_page_buffer();
        let cell_number = self.page_header.cell_number;
        if self.page_header.btree_type == BTreeType::InteriorPage {
            return &buffer[12..12 + cell_number as usize * 2];
        } else {
            return &buffer[8..8 + cell_number as usize * 2];
        }
    }

    // Get a slice
    // This function does not automaticaly shift the offset to after the file header
    // in case of the page is the first page. This functions is used mostly to retrieve record
    pub fn get_slice(&self, start: usize, end: Option<usize>) -> &[u8] {
        if let Some(end_range) = end {
            &self.buffer[start..end_range]
        } else {
            &self.buffer[start..]
        }
    }

    pub fn get_nth_record(&self, index: usize) -> Record<'_> {
        let cell_array = self.get_cell_pointer_array();
        let offset_start = index * 2;
        let mut cursor = Cursor::new(&cell_array[offset_start..]);
        if let Ok(cell_start) = cursor.read_u16::<BigEndian>() {
            Record::new(&self.get_slice(cell_start as usize, None))
                .expect("Error: indexing record, file parsing failed")
        } else {
            panic!("Page error: out of bound index record")
        }
    }
}

#[derive(PartialEq)]
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
