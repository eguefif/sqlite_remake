use std::io::{Cursor, Read};
use byteorder::{ReadBytesExt, BigEndian};
use anyhow::Result;
use crate::types::Varint;

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
    pub col_serial_types: Vec<ColSerialType>
}

impl RecordHeader {
    pub fn new(buffer: &[u8]) -> Self {
        let size = Varint::new(buffer);
        let mut col_serial_types = vec![];
        let mut offset = size.size;
        for _ in 0..size.varint {
            let col_type = Varint::new(&buffer[offset..]);
            
            offset += col_type.size;
            col_serial_types.push(ColSerialType::new(col_type.varint as usize));

        }
        Self {
            size: size.varint as usize,
            col_serial_types

        }
    }
}


pub enum FieldType {
    TNull,
    TI8(i8),
    TI16(i16),
    TI32(i32),
    TI48(i64),
    TI64(i64),
    TF64(f64),
    T0,
    T1,
    TVar,
    TBlob(Vec<u8>),
    TStr(String),
}

impl FieldType {
    pub fn from_col_serial_type(serial_type: &ColSerialType, cursor: &mut Cursor<&[u8]>) -> Result<FieldType> {
        let col = match serial_type {
            ColSerialType::Null => FieldType::TNull,
            ColSerialType::Vu8 => FieldType::TI8(cursor.read_i8()?),
            ColSerialType::Vu16 => FieldType::TI16(cursor.read_i16::<BigEndian>()?),
            ColSerialType::Vu32 => FieldType::TI32(cursor.read_i32::<BigEndian>()?),
            ColSerialType::Vu48 => FieldType::TI48(FieldType::get_i48(cursor)?),
            ColSerialType::Vu64 => FieldType::TI64(cursor.read_i64::<BigEndian>()?),
            ColSerialType::Vf64 => FieldType::TF64(cursor.read_f64::<BigEndian>()?),
            ColSerialType::V0 => FieldType::T0,
            ColSerialType::V1 => FieldType::T1,
            ColSerialType::Variable => FieldType::TVar,
            ColSerialType::Blob(size) => {
                let mut blob = Vec::new();
                blob.resize(*size, 0);
                cursor.read_exact(&mut blob)?;
                FieldType::TBlob(blob)
            }
            ColSerialType::Str(size) => {
                let mut buffer = Vec::new();
                buffer.resize(*size, 0);
                cursor.read_exact(&mut buffer)?;
                FieldType::TStr(String::from_utf8(buffer)?)
            }
        };
        Ok(col)
    }

    pub fn get_i48(_cursor: &mut Cursor<&[u8]>) -> Result<i64> {
        todo!("Handle i48 field type in record")
    }
}

pub struct Record<'a> {
    record_size: usize,
    rowid: usize,
    header: RecordHeader,
    buffer: &'a [u8],
    record_start: usize, // When actual record start, after cell header + record header + rowid
    fields: Vec<FieldType>
    
}

impl<'a> Record<'a> {
    pub fn new(buffer: &'a[u8]) -> Result<Self>{
        let record_size = Varint::new(buffer);
        let rowid = Varint::new(&buffer[record_size.size..]);
        let buffer_start = record_size.size + rowid.size;
        let header = RecordHeader::new(&buffer[buffer_start..]);
        let record_start = record_size.size + rowid.size + header.size;
        let mut fields: Vec<FieldType> = vec![];
        let mut cursor = Cursor::new(&buffer[record_start..]);
        for col_serial_type in header.col_serial_types.iter() {
            let field = FieldType::from_col_serial_type(&col_serial_type, &mut cursor);
            fields.push(field?);

        }
        Ok(Self {
            record_size: record_size.varint as usize,
            rowid: rowid.varint as usize,
            header,
            buffer: buffer,
            record_start,
            fields
        })

    }

    pub fn get_record(&self) -> &[u8] {
        &self.buffer[self.record_start..]
    }
}
