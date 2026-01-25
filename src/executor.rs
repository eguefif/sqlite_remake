use crate::dbmetadata::DBMetadata;
use crate::get_page;
use crate::parser::Query;
use anyhow::{Result, anyhow};
use std::fs::File;
use std::io::BufReader;

pub fn execute(
    query: Query,
    dbinfo: DBMetadata,
    buffer: &mut BufReader<File>,
    page_size: usize,
) -> Result<()> {
    let Some(table) = dbinfo.schema.get(&query.from) else {
        return Err(anyhow!("The table does not exists"));
    };

    let page = get_page(buffer, page_size, table.rootpage)?;

    println!("{}", page.get_record_number());
    Ok(())
}
