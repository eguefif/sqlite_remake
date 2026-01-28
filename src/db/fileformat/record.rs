use crate::db::fileformat::types::Varint;
/// Module to handle Record parsing from a cell payload
/// ! See 2.1 Record Format in https://www.sqlite.org/fileformat.html
/// A record is contains by a Cell.
// TODO: the mapping with RTYpe should be in RType, record should not know a thing about
// the outside world
use crate::executor::db_response::RType;
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Cursor, Read};

#[allow(unused)]
pub struct Record<'a> {
    cell_size: usize,
    rowid: usize,
    header: RecordHeader,
    buffer: &'a [u8],
    record_start: usize, // When actual record start, after cell header + record header + rowid
    pub fields: Vec<FieldType>,
}

impl<'a> Record<'a> {
    pub fn new(buffer: &'a [u8]) -> Result<Self> {
        // Parsing cell Header
        let cell_size = Varint::new(buffer);
        let rowid = Varint::new(&buffer[cell_size.size..]);

        // Parsing record header
        let buffer_start = cell_size.size + rowid.size;
        let header = RecordHeader::new(&buffer[buffer_start..]);
        let record_start = cell_size.size + rowid.size + header.size;
        let mut fields: Vec<FieldType> = vec![];
        let mut cursor = Cursor::new(&buffer[record_start..]);
        for col_serial_type in header.col_serial_types.iter() {
            let field = FieldType::from_col_serial_type(&col_serial_type, &mut cursor);
            fields.push(field?);
        }
        Ok(Self {
            cell_size: cell_size.varint as usize,
            rowid: rowid.varint as usize,
            header,
            buffer: buffer,
            record_start,
            fields,
        })
    }

    pub fn get_record(&self) -> &[u8] {
        &self.buffer[self.record_start..]
    }

    pub fn get_col(&self, index: usize) -> RType {
        RType::from_fieldtype(self.fields[index].clone())
    }
}

#[derive(Debug)]
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
    Str(usize),
}

// Represents all the type that a record collumns can be.
// See 2.1 Record Format in https://www.sqlite.org/fileformat.html
// We don't store the value of the type yet.
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

    pub fn size(&self) -> usize {
        match *self {
            ColSerialType::Null => 0,
            ColSerialType::Vu8 => 1,
            ColSerialType::Vu16 => 2,
            ColSerialType::Vu32 => 4,
            ColSerialType::Vu48 => 6,
            ColSerialType::Vu64 => 8,
            ColSerialType::Vf64 => 8,
            ColSerialType::V0 => 0,
            ColSerialType::V1 => 0,
            ColSerialType::Variable => panic!("Got variable col type in Record"),
            ColSerialType::Blob(size) => size,
            ColSerialType::Str(size) => size,
        }
    }
}

// RecordHeader allows us to know how to find values in the record
#[derive(Debug)]
pub struct RecordHeader {
    size: usize,
    pub col_serial_types: Vec<ColSerialType>,
}

impl RecordHeader {
    pub fn new(buffer: &[u8]) -> Self {
        let size = Varint::new(buffer);
        let mut col_serial_types = vec![];
        let mut offset = size.size;
        loop {
            let col_type = Varint::new(&buffer[offset..]);

            offset += col_type.size;
            col_serial_types.push(ColSerialType::new(col_type.varint as usize));
            if offset == size.varint as usize {
                break;
            }
        }
        Self {
            size: size.varint as usize,
            col_serial_types,
        }
    }
}

// FieldType represernts the type and store the value of the column
#[derive(Debug, Clone)]
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
    pub fn from_col_serial_type(
        serial_type: &ColSerialType,
        cursor: &mut Cursor<&[u8]>,
    ) -> Result<FieldType> {
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
