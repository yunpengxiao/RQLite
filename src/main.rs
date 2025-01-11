mod page;
mod utils;

use anyhow::{bail, Result};
use page::{FileHeader, PageHeader, RowReader};
use std::fs::File;

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
    match command.as_str() {
        ".dbinfo" => {
            let mut file = File::open(&args[1])?;
            let file_header = FileHeader::from(&mut file);
            let page_header = PageHeader::from(&mut file);

            println!("database page size: {}", file_header.page_size);
            println!("number of tables: {}", page_header.cell_count);
        }
        ".tables" => {
            let mut file = File::open(&args[1])?;
            let num_of_cell = PageHeader::from(&mut file).cell_count;
            let row_reader = RowReader::from(&mut file, num_of_cell as usize).unwrap();
            for n in 0..num_of_cell {
                println!("table{n}'s name is {}", row_reader.read(n.into()).2);
            }
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}
