//! Module that contains the sqlite page structure
//!
//! A page can be of different types, see [PageType] enum.
//!
//! The first page is a special page as it contains the database header. It is stored
//! To write
//!
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

use crate::{
    db::{fileformat::record::Record, table::Table},
    executor::db_response::RType,
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
    fn get_cell_pointer_array(&self, start: Option<usize>) -> &[u8] {
        let start = (start.unwrap_or(0) * 2) + 12;
        let buffer = self.get_page_buffer();
        let cell_number = self.cell_number;
        return &buffer[start..start + cell_number as usize * 2];
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

    pub fn get_next_page_pointer<'a>(
        &self,
        where_clause: &Where,
        table: &'a Table,
    ) -> Result<(usize, Option<Vec<usize>>)> {
        //print_page(&self.buffer);
        let cell_array = self.get_cell_pointer_array(None);
        let mut cursor = Cursor::new(cell_array);

        let less_than_where = Where::from_where(where_clause, Token::LT);

        let mut rowids = vec![];
        for i in 0..self.cell_number {
            let cell_offset = cursor.read_u16::<BigEndian>()? as usize;
            let mut cell_buffer = Cursor::new(&self.buffer[cell_offset as usize..]);
            let pointer_page = cell_buffer.read_u32::<BigEndian>()? as usize;

            let mut record = Record::new(&self.get_slice(cell_offset + 4, None), table, false)?;

            let rowid = get_rowid(&mut record);
            let column = where_clause.get_identifier().unwrap();
            let field = record.take_field(column);
            if field == Some(RType::Null) {
                continue;
            }
            if compare(where_clause, field.as_ref()) == true {
                rowids.push(rowid);
                if self.check_next(where_clause, i, table) == false {
                    return Ok((pointer_page, Some(rowids)));
                }
            } else if compare(&less_than_where, field.as_ref()) == true {
                return Ok((pointer_page, None));
            }
        }
        Ok((self.right_most_pointer, None))
    }

    fn check_next(&self, where_clause: &Where, cell_number: usize, table: &Table) -> bool {
        let cell_array = self.get_cell_pointer_array(Some(cell_number));
        let mut cursor = Cursor::new(cell_array);
        let Ok(cell_offset) = cursor.read_u16::<BigEndian>() else {
            return false;
        };
        let mut cell_buffer = Cursor::new(&self.buffer[cell_offset as usize..]);
        let _ = cell_buffer.read_u32::<BigEndian>();

        let Ok(mut record) = Record::new(
            &self.get_slice(cell_offset as usize + 4, None),
            table,
            false,
        ) else {
            return false;
        };

        let column = where_clause.get_identifier().unwrap();
        let field = record.take_field(column);
        compare(where_clause, field.as_ref())
    }
}

fn compare(where_clause: &Where, field: Option<&RType>) -> bool {
    if where_clause.evaluate(field) == true {
        return true;
    }
    false
}

fn get_rowid(record: &mut Record) -> usize {
    let field = record
        .take_field("rowid")
        .expect("There iw always a rowid field");
    if let RType::Num(rowid) = field {
        return rowid as usize;
    }
    panic!("Should always have a rowid of type RTYpe::Num");
}

#[allow(dead_code)]
fn print_page(buffer: &[u8]) {
    let mut offset = 0;
    let mut iter = buffer.iter();
    let mut last_row: Vec<&u8> = vec![];
    let mut flag = true;
    'outer: loop {
        let mut row = vec![];
        for _ in 0..16 {
            let Some(byte) = iter.next() else {
                break 'outer;
            };
            row.push(byte);
        }
        if last_row == row {
            if flag {
                println!("*");
                flag = false;
            }
        } else {
            print_offset(offset);
            flag = true;

            for (i, byte) in row.iter().enumerate() {
                print!("{:0<2x?} ", byte);
                if (i + 1) % 8 == 0 {
                    print!("    ");
                }
                offset += 1;
            }
            print!("  |");
            for byte in row.iter() {
                if byte.is_ascii_alphanumeric() {
                    print!("{}", **byte as char);
                } else {
                    print!(".");
                }
            }
            print!("|");
            println!("");
            last_row = row;
        }
    }
    println!("");
}

fn print_offset(offset: usize) {
    print!("{:0>5x} : ", offset)
}
