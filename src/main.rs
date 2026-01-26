use crate::db::DB;
use anyhow::{Result, bail};

pub mod db;
pub mod fileformat;
pub mod parser;

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
    let mut db = DB::new(&args[1])?;
    match command.as_str() {
        ".dbinfo" => {
            db.metadata.print_metadata();
        }
        ".tables" => {
            db.metadata.print_table_names();
        }
        _ => {
            let responses = db.process_query(command.to_string())?;
            for response in responses {
                for row in response {
                    for col in row {
                        print!("{}", col);
                    }
                    println!("");
                }
            }
        }
    }

    Ok(())
}
