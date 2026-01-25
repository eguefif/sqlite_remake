use crate::page::Page;
use std::io::Cursor;
use byteorder::{ReadBytesExt, BigEndian};
use crate::types::read_varint;

pub struct DBMetadata {
    page: Page,
    //header: [u8; 100],
    //page_header: [u8; 12],
    schema: SchemaTable,
}

impl DBMetadata {
    pub fn new(page: Page) -> Self {
        Self { page }
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

    fn get_number_of_table(&self) -> u16 {
        // The number of table is the number of cell on Page1. Because of the file header
        // the page header is stored at offset 100
        self.page.page_header.cell_number
    }

    pub fn print_table_names(&self) {
        let cell_array = self.page.get_cell_pointer_array();
        let mut cursor = Cursor::new(cell_array);
        let table_number = self.get_number_of_table() as usize;
        for _ in 0..table_number {
            // TODO: read cell size / ignore id
            // read record header.
            //    byte1 => header size including this byte
            //    each byte is a code to represent the type of one column
            //    When we know the  type of each columns, we know the size of one record
            //    we get record by record and extract, using the type, the right column
            let record_header_start = cursor.read_u16::<BigEndian>().expect("We know the numbrer of table") as usize;
            let cell_header = self.page.get_slice(record_header_start, record_header_start + 10);
            let (record_size, varint_size) = read_varint(cell_header);
            let _rowid = cell_header[varint_size];
            let record_header_start = record_header_start + varint_size + 1;
            let record =  self.page.get_slice(record_header_start, record_header_start + record_size as usize);
            self.schema.push(Table::new(&record));            

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

pub enum ColType {

}

pub struct Table {
    table_type: TableType,
    name: String,
    tablename: String,
    rootpage: usize,
    tabledef: String,
    columns: Vec<ColType>
}

impl Table {
    fn new(record: &[u8]) -> Self {
        let (record_header_size, varint_size) = read_varint(record);
        let mut columns : Vec<ColType> = vec![];
        Self {
            columns,
        }
    }
}
