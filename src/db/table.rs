use crate::fileformat::record::FieldType;
use crate::fileformat::record::Record;
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
    pub tablename: String,
    pub rootpage: usize,
    tabledef: String,
    cols_name: Vec<String>,
}

impl Table {
    pub fn new(record: &mut Record) -> Self {
        let FieldType::TStr(ref table_type) = record.fields[0] else {
            panic!("Wrong type table type schema")
        };
        let FieldType::TStr(ref name) = record.fields[1] else {
            panic!("Wrong type name schema")
        };
        let FieldType::TStr(ref tablename) = record.fields[2] else {
            panic!("Wrong type tablename schema")
        };
        let FieldType::TStr(ref tabledef) = record.fields[4] else {
            panic!("Wrong type tabledef")
        };

        let cols_name = Table::get_cols_name(tabledef);

        Self {
            table_type: TableType::from_str(&table_type),
            name: name.to_string(),
            tablename: tablename.to_string(),
            rootpage: Table::get_root_page(record.fields[3].clone()),
            tabledef: tabledef.to_string(),
            cols_name,
        }
    }

    fn get_root_page(record: FieldType) -> usize {
        match record {
            FieldType::TNull => panic!("Table parsing: this type cannot be used for root_page"),
            FieldType::TI8(num) => num as usize,
            FieldType::TI16(num) => num as usize,
            FieldType::TI32(num) => num as usize,
            FieldType::TI48(num) => num as usize,
            FieldType::TI64(num) => num as usize,
            FieldType::TF64(num) => num as usize,
            FieldType::T0 => 0,
            FieldType::T1 => 1,
            FieldType::TVar => panic!("Table parsing: this type cannot be used for root_page"),
            FieldType::TBlob(_) => panic!("Table parsing: this type cannot be used for root_page"),
            FieldType::TStr(_) => panic!("Table parsing: this type cannot be used for root_page"),
        }
    }

    fn get_cols_name(tabledef: &str) -> Vec<String> {
        let values_str = tabledef.split('(').collect::<Vec<_>>()[1];
        values_str
            .split(',')
            .map(|value| Table::trim_column_def(value.trim()))
            .collect::<Vec<_>>()
    }

    fn trim_column_def(value: &str) -> String {
        if value.contains(' ') {
            value.split(' ').next().unwrap().trim().to_string()
        } else {
            value.trim().to_string()
        }
    }

    pub fn get_col_index(&self, col_name: &str) -> usize {
        self.cols_name
            .iter()
            .position(|name| col_name == name)
            .unwrap()
    }
}
