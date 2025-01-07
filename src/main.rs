use anyhow::{bail, Result};
use std::fs::File;
use std::io::prelude::*;
use std::os::unix::prelude::FileExt;

const FILE_HEADER_SIZE: usize = 100;

struct FileHeader {
    page_size: u16,
}

impl FileHeader {
    fn from(file: &mut File) -> Self {
        let mut header = [0; FILE_HEADER_SIZE];
        let _ = file.read_exact(&mut header);
        Self {
            page_size: u16::from_be_bytes([header[16], header[17]]),
        }
    }
}

struct PageHeader {
    //page_type: u8,
    //start_of_freeblock: u16,
    cell_count: u16,
    //start_of_cell_content: u16,
    //fragmented_bytes: u8,
    //page_number: u32,
}

impl PageHeader {
    fn from(file: &mut File) -> Self {
        let mut buffer = [0; 2];
        let _bytes_read = file.read_at(&mut buffer, 103);
        Self {
            cell_count: u16::from_be_bytes([buffer[0], buffer[1]]),
        }
    }
}

struct CellArray {

}

struct Cell {
    
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
