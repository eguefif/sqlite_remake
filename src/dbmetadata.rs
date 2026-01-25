use crate::page::Page;
use crate::table::{SchemaTable, Table};
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
            let table = Table::new(&mut record);
            schema.insert(table.tablename.clone(), table);
        }
        schema
    }

    pub fn print(&self) {
        println!("database page size: {}", self.get_page_size());
        println!("number of tables: {}", self.get_number_of_table());
    }

    fn get_page_size(&self) -> u16 {
        // The page size is stored at the 16th byte offset, using 2 bytes in big-endian order
        if let Some(header) = self.page.get_db_header() {
            println!("{:?}", header);
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
    pub fn print_table_names(self) {
        let mut tablenames = Vec::new();
        for table in self.schema.values() {
            tablenames.push(&table.tablename)
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
