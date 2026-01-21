use crate::dbinfo::DBMetadata;
use crate::page::Page;
use anyhow::{Result, bail};
use std::fs::File;
use std::io::{BufReader, Read, Seek};

pub mod dbinfo;
pub mod page;
pub mod types;

fn main() -> Result<()> {
    // Parse arguments
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        0 | 1 => bail!("Missing <database path> and <command>"),
        2 => bail!("Missing <command>"),
        _ => {}
    }

    // Parse command and act accordingly
    let command = &args[2];
    let file = File::open(&args[1])?;
    let mut buf_reader = BufReader::new(file);
    let page_size = get_page_size(&mut buf_reader)?;

    let dbinfo = get_dbinfo(&mut buf_reader, page_size)?;
    match command.as_str() {
        ".dbinfo" => {
            dbinfo.print();
        }
        ".tables" =>  {
            dbinfo.print_table_names();
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}

fn get_dbinfo(buf_reader: &mut BufReader<File>, page_size: u16) -> Result<DBMetadata> {
    let mut buffer = Vec::new();
    buffer.resize(page_size as usize, 0);
    buf_reader.read_exact(&mut buffer)?;
    let page1 = Page::new(buffer, 1)?;
    Ok(DBMetadata::new(page1))
}

fn get_page_size(buf_reader: &mut BufReader<File>) -> Result<u16> {
    let mut header: [u8; 100] = [0; 100];
    buf_reader.read_exact(&mut header)?;
    buf_reader.rewind()?;
    Ok(u16::from_be_bytes([header[16], header[17]]))
}
