use anyhow::{Result, bail};
use std::fs::File;
use std::io::prelude::*;
use crate::dbinfo::DBInfo;

pub mod dbinfo;

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
    let mut file = File::open(&args[1])?;
    match command.as_str() {
        ".dbinfo" => {
            let mut header = [0; 112];
            file.read_exact(&mut header)?;
            let dbinfo = DBInfo::new(&header);
            dbinfo.print();

        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}
