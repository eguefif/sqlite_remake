use anyhow::{Result, bail};
use codecrafters_sqlite::db::{DB, db_response::Response, parser::statement::Statement};

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
    if let Some(response) = db.execute(command)? {
        display_response(&response);
    }

    Ok(())
}

fn display_response(responses: &[(Statement, Response)]) {
    for (_, response) in responses {
        for row in response {
            for (i, col) in row.iter().enumerate() {
                if i != 0 {
                    print!("|");
                }
                print!("{}", col);
            }
            println!("");
        }
    }
}
