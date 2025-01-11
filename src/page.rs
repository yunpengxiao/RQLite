use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::os::unix::prelude::FileExt;

use crate::utils::read_variant;

/*
    A b-tree page is divided into regions in the following order:
        * The 100-byte database file header (found on page 1 only)
        * The 8 or 12 byte b-tree page header
        * The cell pointer array
        * Unallocated space
        * The cell content area
        * The reserved region
*/
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
    const MAX_PAGE_HEADER_SIZE: usize = 8;

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

pub struct RowReader {
    pub pointers: Vec<u16>,
    pub cells: Vec<Cell>,
}

impl RowReader {
    const BUFFER_SIZE: usize = 1000;

    pub fn from(file: &mut File, cell_count: usize) -> io::Result<Self> {
        let mut buffer = vec![0; cell_count * 2];
        let mut cell_pointers = Vec::new();
        // Page header size can be 12 bytes too, just use 8 here for simplicity
        file.read_exact_at(
            &mut buffer[..],
            u64::try_from(FileHeader::FILE_HEADER_SIZE + PageHeader::MAX_PAGE_HEADER_SIZE).unwrap(),
        )?;
        for slice in buffer.as_slice().chunks(2) {
            let offset = u16::from_be_bytes(slice.try_into().unwrap());
            cell_pointers.push(offset);
        }

        let mut cells: Vec<Cell> = Vec::new();
        for cell_location in &cell_pointers {
            let mut buffer = [0; Self::BUFFER_SIZE];
            let _ = file.read_exact_at(&mut buffer, (*cell_location) as u64);
            cells.push(Cell::from(&buffer));
        }

        Ok(Self {
            pointers: cell_pointers,
            cells,
        })
    }

    pub fn read(&self, row_num: u32) -> (String, String, String) {
        (
            self.cells[row_num as usize].record.get_column(0),
            self.cells[row_num as usize].record.get_column(1),
            self.cells[row_num as usize].record.get_column(2)
        )
    }
}

pub struct Cell {
    pub size_of_record: usize,
    pub rowid: i64,
    pub record: Record,
}

impl Cell {
    pub fn from(data: &[u8]) -> Self {
        use crate::utils::read_variant;

        let (size_of_record, bytes_read1) = read_variant(data);
        let (rowid, bytes_read2) = read_variant(&data[bytes_read1..]);
        let record = Record::from(&data[bytes_read1 + bytes_read2..]);
        Self {
            size_of_record: size_of_record.try_into().unwrap(),
            rowid,
            record,
        }
    }
}

pub enum SerialType {
    String,
    Blob,
    NULL,
    Integer,
    //Float,
}

pub struct Record {
    pub columns: Vec<(SerialType, Vec<u8>)>,
}

impl Record {
    pub fn from(data: &[u8]) -> Self {
        let (record_head_size, first_type_offset) = read_variant(&data[..]);
        //println!("record head size is {record_head_size}");
        let mut column_pointer = record_head_size;
        let mut serial_type_pointer: usize = first_type_offset;
        let mut columns: Vec<(SerialType, Vec<u8>)> = Vec::new();
        while serial_type_pointer != record_head_size as usize {
            let (serial_type, bytes_read) = read_variant(&data[serial_type_pointer..]);
            let size_of_column: i64;
            let st: SerialType;
            if serial_type >= 12 && serial_type % 2 == 0 {
                size_of_column = (serial_type - 12) / 2;
                st = SerialType::Blob;
            } else if serial_type >= 13 && serial_type % 2 != 0 {
                size_of_column = (serial_type - 13) / 2;
                st = SerialType::String;
            } else if serial_type >= 1 && serial_type <= 4 {
                size_of_column = serial_type;
                st = SerialType::Integer;
            } else {
                size_of_column = 0;
                st = SerialType::NULL;
            }

            let value: Vec<u8> = data
                [(column_pointer as usize)..(column_pointer as usize) + (size_of_column as usize)]
                .to_vec();
            columns.push((st, value));
            serial_type_pointer += bytes_read;
            column_pointer += size_of_column;
        }
        Self { columns }
    }

    pub fn get_column(&self, index: usize) -> String {
        let (stype, data) = &self.columns[index];
        match stype {
            SerialType::String => String::from_utf8(data.clone()).unwrap(),
            _ => String::from(" "),
        }
    }
}
