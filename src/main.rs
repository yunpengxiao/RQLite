mod page;

use anyhow::{bail, Result};
use std::fs::File;
use page::{PageHeader, FileHeader};

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
            /*
            A b-tree page is divided into regions in the following order:
                * The 100-byte database file header (found on page 1 only)
                * The 8 or 12 byte b-tree page header
                * The cell pointer array
                * Unallocated space
                * The cell content area
                * The reserved region
            */
            let mut file = File::open(&args[1])?;
            let file_header = FileHeader::from(&mut file);
            let page_header = PageHeader::from(&mut file);

            println!("database page size: {}", file_header.page_size);
            println!("number of tables: {}", page_header.cell_count);
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}