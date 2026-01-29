/// Module to handle Record parsing from a cell payload
/// ! See 2.1 Record Format in https://www.sqlite.org/fileformat.html
/// A record is contains by a Cell.
// TODO: the mapping with RTYpe should be in RType, record should not know a thing about
// the outside world
use crate::db::fileformat::types::Varint;
use crate::executor::db_response::RType;
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Cursor, Read};

#[allow(unused)]
pub struct Record {
    cell_size: usize,
    rowid: usize,
    header: RecordHeader,
    record_start: usize, // When actual record start, after cell header + record header + rowid
    fields: Vec<RType>,
}

impl Record {
    pub fn new(buffer: &[u8]) -> Result<Self> {
        // Parsing cell Header
        let cell_size = Varint::new(buffer);
        let rowid = Varint::new(&buffer[cell_size.size..]);

        // Parsing record header
        let buffer_start = cell_size.size + rowid.size;
        let header = RecordHeader::new(&buffer[buffer_start..]);

        // Parsing record
        let record_start = cell_size.size + rowid.size + header.size;
        let mut fields: Vec<RType> = vec![];
        let mut cursor = Cursor::new(&buffer[record_start..]);
        for col_serial_type in header.col_serial_types.iter() {
            let field = Self::from_col_serial_type(&col_serial_type, &mut cursor);
            fields.push(field?);
        }
        Ok(Self {
            cell_size: cell_size.varint as usize,
            rowid: rowid.varint as usize,
            header,
            record_start,
            fields,
        })
    }
    pub fn take_fields(&mut self) -> Vec<RType> {
        std::mem::take(&mut self.fields)
    }

    /// Move out a value from the record
    pub fn take_field(&mut self) -> RType {
        if self.fields.len() > 0 {
            return self.fields.remove(0);
        }
        panic!("Record: cannot take field anymore, fields.len() == 0",);
    }

    pub fn from_col_serial_type(
        serial_type: &ColSerialType,
        cursor: &mut Cursor<&[u8]>,
    ) -> Result<RType> {
        let col = match serial_type {
            ColSerialType::Null => RType::Null,
            ColSerialType::Vu8 => RType::Num(cursor.read_i8()? as i64),
            ColSerialType::Vu16 => RType::Num(cursor.read_i16::<BigEndian>()? as i64),
            ColSerialType::Vu32 => RType::Num(cursor.read_i32::<BigEndian>()? as i64),
            ColSerialType::Vu48 => RType::Num(Self::get_i48(cursor)?),
            ColSerialType::Vu64 => RType::Num(cursor.read_i64::<BigEndian>()? as i64),
            ColSerialType::Vf64 => RType::Num(cursor.read_f64::<BigEndian>()? as i64),
            ColSerialType::V0 => RType::Num(0),
            ColSerialType::V1 => RType::Num(1),
            ColSerialType::Variable => todo!("TODO: ColSeriableType variable"),
            ColSerialType::Blob(size) => {
                let mut blob = Vec::new();
                blob.resize(*size, 0);
                cursor.read_exact(&mut blob)?;
                RType::Blob(blob)
            }
            ColSerialType::Str(size) => {
                let mut buffer = Vec::new();
                buffer.resize(*size, 0);
                cursor.read_exact(&mut buffer)?;
                RType::Str(String::from_utf8(buffer)?)
            }
        };
        Ok(col)
    }

    pub fn get_i48(_cursor: &mut Cursor<&[u8]>) -> Result<i64> {
        todo!("Handle i48 field type in record")
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

impl FieldType {}
