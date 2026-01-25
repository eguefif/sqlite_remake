use crate::db::dbmetadata::DBMetadata;
use crate::fileformat::page::Page;
use crate::parser::{Parser, query::Query};
use anyhow::{Result, anyhow};
use std::fs::File;
use std::io::BufReader;
use std::io::{Read, Seek, SeekFrom};

pub mod dbmetadata;
pub mod table;

pub struct DB {
    pub metadata: DBMetadata,
    page_size: usize,
    buf_reader: BufReader<File>,
}

impl DB {
    pub fn new(filename: &str) -> Result<Self> {
        let file = File::open(filename)?;
        let mut buf_reader = BufReader::new(file);
        let page_size = DB::get_page_size(&mut buf_reader)? as usize;

        let mut buffer = Vec::new();
        buffer.resize(page_size as usize, 0);
        buf_reader.read_exact(&mut buffer)?;
        buf_reader.rewind()?;
        let page = Page::new(buffer, 1)?;
        let metadata = DBMetadata::new(page);
        Ok(Self {
            metadata,
            page_size,
            buf_reader,
        })
    }

    fn get_page_size(buf_reader: &mut BufReader<File>) -> Result<u16> {
        let mut header: [u8; 100] = [0; 100];
        buf_reader.read_exact(&mut header)?;
        buf_reader.rewind()?;
        Ok(u16::from_be_bytes([header[16], header[17]]))
    }

    fn get_page(&mut self, root_page: usize) -> Result<Page> {
        let mut buffer = Vec::new();
        buffer.resize(self.page_size, 0);
        let offset = ((root_page - 1) * self.page_size) as u64;
        self.buf_reader.seek(SeekFrom::Start(offset))?;
        self.buf_reader.read_exact(&mut buffer)?;
        Page::new(buffer, root_page)
    }

    pub fn process_query(&mut self, query_str: String) -> Result<()> {
        let mut parser = Parser::new(&query_str);
        parser.parse();
        for query in parser.queries {
            self.execute(&query)?;
        }
        Ok(())
    }

    fn execute(&mut self, query: &Query) -> Result<()> {
        let Some(table) = self.metadata.schema.get(&query.from) else {
            return Err(anyhow!("The table does not exists"));
        };

        let page = self.get_page(table.rootpage)?;

        println!("{}", page.get_record_number());
        Ok(())
    }
}
