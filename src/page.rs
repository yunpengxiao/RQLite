use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::os::unix::prelude::FileExt;

use crate::utils::read_variant;

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

    pub fn read_cells(&self, file: &mut File) -> Cell {
        let first_cell_location = self.pointers[0];
        let mut buffer = [0; 150];
        let _ = file.read_exact_at(&mut buffer, first_cell_location.into());
        Cell::from(&buffer)
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
        //let (record_header_size, bytes_read3) = read_variant(&data[bytes_read1 + bytes_read2..]);
        //let (type1, bytes_read4) = read_variant(&data[bytes_read1 + bytes_read2 + bytes_read3..]);
        println!("size is {size_of_record}, rowid is {rowid}");
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
        let mut column_pointer = record_head_size;
        let mut serial_type_pointer = first_type_offset;
        let mut columns: Vec<(SerialType, Vec<u8>)> = Vec::new();
        while serial_type_pointer != record_head_size as usize {
            let (serial_type, bytes_read) = read_variant(&data[serial_type_pointer..]);
            let mut size_of_column = 0;
            let mut st = SerialType::NULL;
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

            let value: Vec<u8> = data[(column_pointer as usize)..(column_pointer as usize) + (size_of_column as usize)].to_vec();
            columns.push((st, value));
            serial_type_pointer += bytes_read;
            column_pointer += size_of_column;
        }
        Self {
            columns,
        }
    }
}