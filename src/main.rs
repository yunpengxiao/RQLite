use anyhow::{bail, Result};
use std::fs::File;
use std::io::prelude::*;
use std::os::unix::fs::FileExt;

fn get_page_header(file: &mut File) -> u16 {
    let mut buffer = vec![0u8; 2];
    let _bytes_read = file.read_at(&mut buffer, 103).unwrap();
    u16::from_be_bytes([buffer[0], buffer[1]])
}

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
            let mut header = [0; 100];
            file.read_exact(&mut header)?;
            /*
            A b-tree page is divided into regions in the following order:
                * The 100-byte database file header (found on page 1 only)
                * The 8 or 12 byte b-tree page header
                * The cell pointer array
                * Unallocated space
                * The cell content area
                * The reserved region
            */
            // The page size is stored at the 16th byte offset, using 2 bytes in big-endian order
            #[allow(unused_variables)]
            let page_size = u16::from_be_bytes([header[16], header[17]]);
            let cell_count = get_page_header(&mut file);
            // You can use print statements as follows for debugging, they'll be visible when running tests.
            eprintln!("Logs from your program will appear here!");

            // Uncomment this block to pass the first stage
            println!("database page size: {}", page_size);
            println!("number of tables: {}", cell_count);
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}
