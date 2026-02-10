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
    pub table_type: TableType,
    name: String,
    pub tablename: String,
    root_page: usize,
    tabledef: String,
    pub cols_name: Vec<String>,
    pub indexes: Vec<Table>,
}

impl Table {
    pub fn new(
        table_type: String,
        name: String,
        tablename: String,
        rootpage: usize,
        tabledef: String,
        cols_name: Vec<String>,
    ) -> Self {
        Self {
            table_type: TableType::from_str(&table_type),
            name,
            tablename,
            root_page: rootpage,
            tabledef: tabledef,
            cols_name,
            indexes: vec![],
        }
    }

    pub fn schema_table() -> Self {
        Self {
            table_type: TableType::from_str("table"),
            name: "Schema".to_string(),
            tablename: "Schema".to_string(),
            root_page: 0,
            tabledef: "".to_string(),
            cols_name: vec![
                "table_type".to_string(),
                "name".to_string(),
                "tablename".to_string(),
                "rootpage".to_string(),
                "tabledef".to_string(),
            ],
            indexes: vec![],
        }
    }

    pub fn get_column_name(&self, index: usize) -> &str {
        return &self.cols_name[index];
    }

    pub fn get_root_page(&self) -> usize {
        self.root_page
    }
}
