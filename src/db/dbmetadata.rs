//! This module offer an abstraction over the sqlite database metadata
//!
use crate::db::fileformat::page::Page;
use crate::db::table::{SchemaTable, Table};
use crate::executor::db_response::RType;
use std::collections::HashMap;

pub struct DBMetadata {
    page: Page,
    pub schema: SchemaTable,
}

impl DBMetadata {
    pub fn new(page: Page) -> Self {
        let schema = Self::create_table_schema(&page);
        Self { page, schema }
    }

    fn create_table_schema(page: &Page) -> SchemaTable {
        let mut schema: SchemaTable = HashMap::new();
        for n in 0..page.get_record_number() {
            let mut record = page.get_nth_record(n);
            let RType::Str(table_type) = record.take_field() else {
                panic!("Wrong type table type schema")
            };
            let RType::Str(name) = record.take_field() else {
                panic!("Wrong type name schema")
            };
            let RType::Str(tablename) = record.take_field() else {
                panic!("Wrong type tablename schema")
            };
            let rootpage = Self::get_root_page(record.take_field());
            let RType::Str(tabledef) = record.take_field() else {
                panic!("Wrong type tabledef")
            };

            let cols_name = Self::get_cols_name(&tabledef);

            let table = Table::new(table_type, name, rootpage, tabledef, cols_name);
            schema.insert(tablename.to_string(), table);
        }
        schema
    }

    fn get_cols_name(tabledef: &str) -> Vec<String> {
        let values_str = tabledef.split('(').collect::<Vec<_>>()[1];
        values_str
            .split(',')
            .map(|value| Self::trim_column_def(value.trim()))
            .collect::<Vec<_>>()
    }

    fn trim_column_def(value: &str) -> String {
        if value.contains(' ') {
            value.split(' ').next().unwrap().trim().to_string()
        } else {
            value.trim().to_string()
        }
    }

    fn get_root_page(record: RType) -> usize {
        match record {
            RType::Null => panic!("Table parsing: this type cannot be used for root_page"),
            RType::Num(num) => num as usize,
            RType::Blob(_) => panic!("Table parsing: this type cannot be used for root_page"),
            RType::Str(_) => panic!("Table parsing: this type cannot be used for root_page"),
        }
    }

    pub fn get_table_root_page(&self, tablename: &str) -> Option<usize> {
        let table = self.schema.get(tablename)?;
        Some(table.root_page)
    }

    pub fn print_metadata(&self) {
        println!("database page size: {}", self.get_page_size());
        println!("number of tables: {}", self.get_number_of_table());
    }

    fn get_page_size(&self) -> u16 {
        // The page size is stored at the 16th byte offset, using 2 bytes in big-endian order
        if let Some(header) = self.page.get_db_header() {
            return u16::from_be_bytes([header[16], header[17]]);
        }
        0
    }

    // The number of table is the number of cell on Page1. Because of the file header
    // the page header is stored at offset 100
    fn get_number_of_table(&self) -> usize {
        self.page.get_record_number()
    }

    // Print tablenames in alphabetical order
    pub fn print_table_names(&self) {
        let mut tablenames = Vec::new();
        for (tablename, _) in self.schema.iter() {
            tablenames.push(tablename.to_string())
        }
        tablenames.sort();
        for (i, tablename) in tablenames.iter().enumerate() {
            if i != 0 {
                print!(" ");
            }
            print!("{}", tablename);
        }
    }
}
