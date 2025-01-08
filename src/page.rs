use std::fs::File;
use std::io;
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
    const PAGE_HEADER_SIZE: usize = 12;

    pub fn from(file: &mut File) -> Self {
        let mut buffer = [0; 2];
        let _bytes_read = file.read_at(
            &mut buffer,
            u64::try_from(FileHeader::FILE_HEADER_SIZE + 3).unwrap(),
        );
        Self {
            cell_count: u16::from_be_bytes([buffer[0], buffer[1]]),
        }
    }
}

pub struct CellPointer {
    pub pointers: Vec<u16>,
}

impl CellPointer {
    pub fn from(file: &mut File, cell_count: usize) -> io::Result<Self> {
        let mut buffer = vec![0; cell_count * 2];
        let mut cell_pointers = Vec::new();
        file.read_exact_at(
            &mut buffer[..],
            u64::try_from(FileHeader::FILE_HEADER_SIZE + PageHeader::PAGE_HEADER_SIZE).unwrap(),
        )?;
        for slice in buffer.as_slice().chunks(2) {
            cell_pointers.push(u16::from_be_bytes(slice.try_into().unwrap()));
        }
        Ok(Self {
            pointers: cell_pointers,
        })
    }

    pub fn read_cells(&self, file: &mut File) -> Vec<Cell> {
        let first_cell_location = self.pointers[0];
        let mut buffer = [0; 100];
        Vec::new()
    }
}

pub struct Cell {
    pub size: usize,
    pub rowid: u64,
    pub record: Record,
}

impl Cell {
    pub fn from(data: &[u8]) -> Self {
        use crate::utils::read_variant;
        let (size, bytes_read) = read_variant(data);
    }
}

pub struct Record {
    pub header_size: usize,
    pub types: Vec<u32>,
    pub rows: Vec<String>,
}

impl Record {
    pub fn from(data: &[u8]) -> Self {
    }
}