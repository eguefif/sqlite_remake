use std::io::Cursor;
use anyhow::Result;
use byteorder::{ReadBytesExt, BigEndian};


pub struct Page {
    buffer: Vec<u8>,
    pub page_number: usize,
    pub page_header: PageHeader
}

impl Page {
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
            page_header
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

    pub fn get_cell_pointer_array(&self) -> &[u8] {
        let buffer = self.get_page_buffer();
        if self.page_header.btree_type == BTreeType::InteriorPage {
            return &buffer[12..]
        } else {
            return &buffer[8..]
        }

    }

    pub fn get_slice(&self, start: usize, end: usize) -> &[u8] {
        let buffer = self.get_page_buffer();
        &buffer[start..end]
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
    pub cell_number: u16,
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
            cell_number: cursor.read_u16::<BigEndian>()?,
            start_content: cursor.read_u16::<BigEndian>()? as usize,
            frag_number: cursor.read_u8()?,
            right_most_pointer: cursor.read_u16::<BigEndian>()? as usize,
        })
    }
}
