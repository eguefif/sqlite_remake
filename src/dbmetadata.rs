use crate::page::Page;
use crate::record::FieldType;
use crate::record::Record;

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
        let mut schema: SchemaTable = vec![];
        for n in 0..page.get_record_number() {
            let mut record = page.get_nth_record(n);
            schema.push(Table::new(&mut record));
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

    fn get_number_of_table(&self) -> usize {
        // The number of table is the number of cell on Page1. Because of the file header
        // the page header is stored at offset 100
        self.page.get_record_number()
    }

    pub fn print_table_names(self) {
        for (i, table) in self.schema.iter().enumerate() {
            if i != 0 {
                print!(" ");
            }
            print!("{}", table.tablename);
        }
    }
}

type SchemaTable = Vec<Table>;

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
    rootpage: usize,
    tabledef: String,
}

impl Table {
    fn new(record: &mut Record) -> Self {
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

        Self {
            table_type: TableType::from_str(&table_type),
            name: name.to_string(),
            tablename: tablename.to_string(),
            rootpage: Table::get_root_page(record.fields[3].clone()),
            tabledef: tabledef.to_string(),
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
}
