//! This module offer an abstraction over the sqlite database metadata
//!
use crate::db::fileformat::page::Page;
use crate::db::table::{SchemaTable, Table};
use crate::executor::db_response::{RType, Response};
use anyhow::{Result, anyhow};
use std::collections::HashMap;

pub struct DBMetadata {
    page: Page,
    pub schema: SchemaTable,
}

impl DBMetadata {
    pub fn new(page: Page) -> Result<Self> {
        let schema = Self::create_table_schema(&page)?;
        Ok(Self { page, schema })
    }

    fn create_table_schema(page: &Page) -> Result<SchemaTable> {
        let mut schema: SchemaTable = HashMap::new();
        let schema_table = Table::schema_table();
        for n in 0..page.get_record_number() {
            let mut record = page.get_nth_record(n, &schema_table)?;
            let Some(RType::Str(table_type)) = record.take_field("table_type") else {
                return Err(anyhow!("Wrong type table type schema"));
            };
            let Some(RType::Str(name)) = record.take_field("name") else {
                return Err(anyhow!("Wrong type name schema"));
            };
            let Some(RType::Str(tablename)) = record.take_field("tablename") else {
                return Err(anyhow!("Wrong type tablename schema"));
            };
            let rootpage = Self::get_root_page(record.take_field("rootpage"))?;
            let Some(RType::Str(tabledef)) = record.take_field("tabledef") else {
                return Err(anyhow!("Wrong type tabledef"));
            };

            let cols_name = Self::get_cols_name(&tabledef);

            let table = Table::new(table_type, name, rootpage, tabledef, cols_name);
            schema.insert(tablename.to_string(), table);
        }
        Ok(schema)
    }

    fn get_cols_name(tabledef: &str) -> Vec<String> {
        let values_str = tabledef.split('(').collect::<Vec<_>>()[1];
        values_str
            .split(',')
            .map(|value| Self::trim_column_def(value.trim()))
            .collect::<Vec<_>>()
    }

    fn trim_column_def(value: &str) -> String {
        // NOTE: Why did I do that ?
        if value.contains(' ') {
            value
                .split(' ')
                .next()
                .expect("We know that there is space")
                .trim()
                .to_string()
        } else {
            value.trim().to_string()
        }
    }

    fn get_root_page(record: Option<RType>) -> Result<usize> {
        match record {
            Some(RType::Null) => {
                return Err(anyhow!(
                    "Table parsing: this type cannot be used for root_page"
                ));
            }
            Some(RType::Num(num)) => Ok(num as usize),
            Some(RType::Blob(_)) => {
                return Err(anyhow!(
                    "Table parsing: this type cannot be used for root_page"
                ));
            }
            Some(RType::Str(_)) => {
                return Err(anyhow!(
                    "Table parsing: this type cannot be used for root_page"
                ));
            }
            None => {
                return Err(anyhow!(
                    "Table parsing: this type cannot be used for root_page"
                ));
            }
        }
    }

    pub fn take_table(&mut self, tablename: &str) -> Option<Table> {
        self.schema.remove(tablename)
    }

    pub fn get_metadata(&self) -> Result<Option<Response>> {
        let page_size = vec![
            RType::Str("database page size:".to_string()),
            RType::Num(self.get_page_size() as i64),
        ];
        let table_number = vec![
            RType::Str("number of tables".to_string()),
            RType::Num(self.get_number_of_table() as i64),
        ];
        Ok(Some(vec![page_size, table_number]))
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
    pub fn get_table_names(&self) -> Result<Option<Response>> {
        let mut tablenames = Vec::new();
        for (tablename, _) in self.schema.iter() {
            tablenames.push(RType::Str(tablename.to_string()))
        }
        tablenames.sort();
        Ok(Some(vec![tablenames]))
    }
}
