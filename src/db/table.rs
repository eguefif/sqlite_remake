//! Module for handling database table schemas.
use std::collections::HashMap;

pub type SchemaTable = HashMap<String, Table>;

pub enum TableType {
    Table,
    Index,
    View,
    Trigger,
}

impl TableType {
    pub fn from_str(str: &str) -> Self {
        match str {
            "table" => TableType::Table,
            "index" => TableType::Index,
            "view" => TableType::View,
            "trigger" => TableType::Trigger,
            _ => panic!("Wront table type"),
        }
    }
}

pub enum ColType {}

#[allow(unused)]
pub struct Table {
    table_type: TableType,
    name: String,
    pub rootpage: usize,
    tabledef: String,
    cols_name: Vec<String>,
}

impl Table {
    pub fn new(
        table_type: String,
        name: String,
        rootpage: usize,
        tabledef: String,
        cols_name: Vec<String>,
    ) -> Self {
        Self {
            table_type: TableType::from_str(&table_type),
            name: name,
            rootpage: rootpage,
            tabledef: tabledef,
            cols_name,
        }
    }

    pub fn get_col_index(&self, col_name: &str) -> usize {
        self.cols_name
            .iter()
            .position(|name| col_name == name)
            .unwrap()
    }
}
