pub struct DBInfo<'a> {
    page: &'a [u8]
}

impl<'a> DBInfo<'a> {
    pub fn new(page: &'a[u8]) -> Self {
        Self {
            page
        }
    }

    pub fn print(&self) {
        println!("database page size: {}", self.get_page_size());
        println!("number of tables: {}", self.get_number_of_table());
    }

    fn get_page_size(&self) -> u16 {
        // The page size is stored at the 16th byte offset, using 2 bytes in big-endian order
        u16::from_be_bytes([self.page[16], self.page[17]])

    }

    fn get_number_of_table(&self) -> u16 {
        // The number of table is the number of cell on Page1. Because of the file header
        // the page header is stored at offset 100
        u16::from_be_bytes([self.page[103], self.page[104]])
    }
}
