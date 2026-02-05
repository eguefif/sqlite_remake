use anyhow::{Result, bail};
use codecrafters_sqlite::{
    db::DB,
    executor::{Executor, db_response::Response},
    parser::statement::Statement,
};

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
    let db = match DB::new(&args[1]) {
        Ok(db) => db,
        Err(error) => bail!("Impossible to read database metadata: {}", error),
    };
    let mut executor = Executor::new(db);
    match executor.execute(command) {
        Ok(response) => display_response(&response),
        Err(e) => eprintln!("Error: {}", e),
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
