use std::fs::File;
use std::io::prelude::*;
use std::os::unix::prelude::FileExt;

pub struct FileHeader {
    pub page_size: u16,
}

impl FileHeader {
    const FILE_HEADER_SIZE: usize = 100;

    pub fn from(file: &mut File) -> Self {
        let mut header = [0; Self::FILE_HEADER_SIZE];
        let _ = file.read_exact(&mut header);
        Self {
            page_size: u16::from_be_bytes([header[16], header[17]]),
        }
    }
}

pub struct PageHeader {
    //page_type: u8,
    //start_of_freeblock: u16,
    pub cell_count: u16,
    //start_of_cell_content: u16,
    //fragmented_bytes: u8,
    //page_number: u32,
}

impl PageHeader {
    pub fn from(file: &mut File) -> Self {
        let mut buffer = [0; 2];
        let _bytes_read = file.read_at(&mut buffer, 103);
        Self {
            cell_count: u16::from_be_bytes([buffer[0], buffer[1]]),
        }
    }
}
