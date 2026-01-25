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
        let cell_number = self.page_header.cell_number;
        if self.page_header.btree_type == BTreeType::InteriorPage {
            return &buffer[12..12 + cell_number as usize * 2]
        } else {
            return &buffer[8..8 + cell_number as usize * 2]
        }

    }

    pub fn get_slice(&self, start: usize, end: usize) -> &[u8] {
        &self.buffer[start..end]
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

pub enum ColSerialType {
    Null,
    Vu8,
    Vu16,
    Vu32,
    Vu48,
    Vu64,
    Vf64,
    V0,
    V1,
    Variable,
    Blob(usize),
    Str(usize)
}

// TODO: Refactor Varint => use new struct
// Impl ColserialType
// Impl RecordHeader
// Impl Record
impl ColSerialType {
    pub fn new(serial_type: usize) -> ColSerialType {
        match serial_type {
            0 => ColSerialType::Null,
            1 => ColSerialType::Vu8,
            2 => ColSerialType::Vu16,
            3 => ColSerialType::Vu32,
            4 => ColSerialType::Vu48,
            5 => ColSerialType::Vu64,
            6 => ColSerialType::Vf64,
            7 => ColSerialType::V0,
            8 => ColSerialType::V1,
            10 | 11 => ColSerialType::Variable,
            _ => {
                if serial_type >= 12 && serial_type % 2 == 0 {
                    let size = (serial_type - 12) / 2;
                    return ColSerialType::Blob(size);
                } else if serial_type > 13 && serial_type % 2 != 0 {
                    let size = (serial_type - 13) / 2;
                    return ColSerialType::Str(size);
                }
                panic!("Error: serial type is not valid");
            }
        }
    }
}

pub struct RecordHeader {
    size: usize,
    col_serial_type: Vec<ColSerialType>
}

impl RecordHeader {
    pub fn new(buffer: &[u8]) -> Self {
        // Get header size: first varint
        // get ech column type => size - 1. It's all varint
        // use enum on each varint
        let size = read_varint(buffer);
        let mut col_serial_type = vec![];
        for _ in 
        Self {
            size,
            col_serial_type

        }
    }
}

pub struct Record<'a> {
    record_size: usize,
    rowid: usize,
    header: RecordHeader,
    buffer: &'a [u8],
    record_start: usize // When actual record start, after cell header + record header + rowid
    
}

impl Record<'a> {
    pub fn new(buffer: &'a[u8]) -> Self<'a> {
        // Get the record_size varint
        // Get the rowid varint
        // get the header => header start depend on varint sizes
        // get the record starts => depends on header size and all varints
        let record_size

    }
}
