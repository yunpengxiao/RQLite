use anyhow::{bail, Result};
use std::fs::File;
use std::io::prelude::*;

const FILE_HEADER_SIZE: usize = 100;

struct FileHeader {
    raw: Vec<u8>,
    cell_count: u16,
}

impl FileHeader {
    fn from(file: &mut File) -> Self {
        let mut header = [0; FILE_HEADER_SIZE];
        let _ = file.read_exact(&mut header);
        Self {
            raw: header.to_vec(),
            cell_count: u16::from_be_bytes([header[3], header[4]]),
        }
    }
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
            let file_header = FileHeader::from(&mut file);
            // You can use print statements as follows for debugging, they'll be visible when running tests.
            eprintln!("Logs from your program will appear here!");

            // Uncomment this block to pass the first stage
            println!("database page size: {}", page_size);
            println!("number of tables: {}", file_header.cell_count);
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}
