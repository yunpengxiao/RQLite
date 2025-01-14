use std::fmt::Display;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use thiserror::Error;

use crate::utils::read_variant;
use crate::utils::MyCoolArrayStuff;

// You need to set RUST_LIB_BACKTRACE=1 to enable backtrace here.
// Running the code like "RUST_LIB_BACKTRACE=1 cargo run -- sample.db tables"
#[derive(Debug, Error)]
pub enum MyError {
    #[error("Io Error: {0}")]
    Io(#[from] std::io::Error, std::backtrace::Backtrace),

    #[error("Offset Error: {0}, at {1}")]
    Offset(#[from] std::num::TryFromIntError, std::backtrace::Backtrace),

    #[error("utf8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error, std::backtrace::Backtrace),

    #[error("Slice error: {0}")]
    Slice(#[from] std::array::TryFromSliceError, std::backtrace::Backtrace),
}

type Result<T> = core::result::Result<T, MyError>;

/*
    A b-tree page is divided into regions in the following order:
        * The 100-byte database file header (found on page 1 only)
        * The 8 or 12 byte b-tree page header
        * The cell pointer array
        * Unallocated space
        * The cell content area
        * The reserved region
*/

#[derive(Debug)]
pub struct PageReader {
    pub file_header: Option<FileHeader>,
    pub page_header: PageHeader,
    pub row_reader: RowReader,
    pub page_num: usize,
}

impl PageReader {
    pub fn from (file: &mut File, page_num: usize) -> Self {
        if page_num == 1 {
            Self {
                file_header: Some(FileHeader::from(file).unwrap()),
                page_header: PageHeader::from(file).unwrap(),
                row_reader: RowReader::from(file).unwrap(),
                page_num,
            }
        } else {
            Self {
                file_header: None,
                page_header: PageHeader::from(file).unwrap(),
                row_reader: RowReader::from(file).unwrap(),
                page_num,
            }
        }   
    }

    pub fn get_page_size(&self) -> u16 {
        self.file_header.as_ref().unwrap().page_size
    }

    pub fn table_count(&self) -> u16 {
        self.page_header.cell_count
    }

    pub fn get_table_names(&self) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        let num_of_cell = self.row_reader.pointers.len();
        for n in 0..num_of_cell {
            result.push(self.row_reader.read(n as u32)[1].to_string());
        }
        result
    }
}

#[derive(Debug)]
struct FileHeader {
    pub page_size: u16,
}

impl FileHeader {
    const FILE_HEADER_SIZE: usize = 100;

    pub fn from(file: &mut File) -> Result<Self> {
        let mut header = [0; Self::FILE_HEADER_SIZE];

        file.read_exact(&mut header)?;

        Ok(Self {
            page_size: u16::from_be_bytes([header[16], header[17]]),
        })
    }
}

#[derive(Debug)]
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

    pub fn from(file: &mut File) -> Result<Self> {
        let mut buffer = [0; 2];
        file.seek(SeekFrom::Start(u64::try_from(FileHeader::FILE_HEADER_SIZE + 3)?))?;
        file.read_exact(&mut buffer)?;
        Ok(Self {
            cell_count: u16::from_be_bytes([buffer[0], buffer[1]]),
        })
    }
}

#[derive(Debug)]
struct RowReader {
    pub pointers: Vec<u16>,
    pub cells: Vec<Cell>,
}

impl RowReader {
    const BUFFER_SIZE: usize = 1000;

    pub fn from(file: &mut File) -> Result<Self> {
        let cell_count = Self::get_cell_count(file).unwrap();
        let mut buffer = vec![0; (cell_count as usize) * 2];
        let mut cell_pointers = Vec::new();
        // Page header size can be 12 bytes too, just use 8 here for simplicity
        file.seek(SeekFrom::Start(u64::try_from(FileHeader::FILE_HEADER_SIZE + PageHeader::MAX_PAGE_HEADER_SIZE)?))?;
        file.read_exact(&mut buffer[..])?;
        for arr in buffer.as_slice().as_array_iter() {
            let offset = u16::from_be_bytes(*arr);
            cell_pointers.push(offset);
        }

        let mut cells: Vec<Cell> = Vec::new();
        for cell_location in &cell_pointers {
            let mut buffer = [0; Self::BUFFER_SIZE];
            file.seek(SeekFrom::Start((*cell_location) as u64))?;
            let _ = file.read_exact(&mut buffer);
            cells.push(Cell::from(&buffer)?);
        }

        Ok(Self {
            pointers: cell_pointers,
            cells,
        })
    }

    pub fn read(&self, row_num: u32) -> Vec<&SerialType> {
        self.cells[row_num as usize].record.columns.iter().collect()
    }

    fn get_cell_count(file: &mut File) -> Result<u16> {
        let mut buffer = [0; 2];
        file.seek(SeekFrom::Start(u64::try_from(FileHeader::FILE_HEADER_SIZE + 3)?))?;
        file.read_exact(&mut buffer)?;
        Ok(u16::from_be_bytes([buffer[0], buffer[1]]))
    }

}

#[derive(Debug)]
struct Cell {
    pub size_of_record: usize,
    pub rowid: i64,
    pub record: Record,
}

impl Cell {
    pub fn from(data: &[u8]) -> Result<Self> {
        use crate::utils::read_variant;

        let (size_of_record, bytes_read1) = read_variant(data);
        let (rowid, bytes_read2) = read_variant(&data[bytes_read1..]);
        let record = Record::from(&data[bytes_read1 + bytes_read2..])?;

        Ok(Self {
            size_of_record: size_of_record.try_into()?,
            rowid,
            record,
        })
    }
}

#[derive(Debug)]
enum SerialType {
    String(String),
    Blob(Vec<u8>),
    NULL,
    Integer(i64),
    //Float,
}

impl Display for SerialType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            SerialType::String(s) => write!(f, "{s}"),
            SerialType::Blob(b) => write!(f, "{b:?}"),
            SerialType::Integer(i) => write!(f, "{i:?}"),
            SerialType::NULL => write!(f, "NULL"),
        }
    }
}

#[derive(Debug)]
struct Record {
    pub columns: Vec<SerialType>,
}

impl Record {
    pub fn from(data: &[u8]) -> Result<Self> {
        let (record_head_size, first_type_offset) = read_variant(&data[..]);
        let mut column_pointer = record_head_size;
        let mut serial_type_pointer: usize = first_type_offset;
        let mut columns: Vec<SerialType> = Vec::new();
        while serial_type_pointer != record_head_size as usize {
            let (serial_type, bytes_read) = read_variant(&data[serial_type_pointer..]);
            let size_of_column: i64;
            let st: SerialType;
            if serial_type >= 12 && serial_type % 2 == 0 {
                size_of_column = (serial_type - 12) / 2;
                st = SerialType::Blob(data
                    [(column_pointer as usize)..(column_pointer as usize) + (size_of_column as usize)]
                    .to_vec());
            } else if serial_type >= 13 && serial_type % 2 != 0 {
                size_of_column = (serial_type - 13) / 2;
                st = SerialType::String(String::from_utf8(data
                    [(column_pointer as usize)..(column_pointer as usize) + (size_of_column as usize)]
                    .to_vec())?);
            } else if serial_type >= 1 && serial_type <= 4 {
                size_of_column = serial_type;
                let cp = column_pointer as usize;
                let cp_end = cp + size_of_column as usize;
                st = match serial_type {
                    1 => SerialType::Integer(i8::from_be_bytes(data[cp..cp_end].try_into()?).into()),
                    2 => SerialType::Integer(i16::from_be_bytes(data[cp..cp_end].try_into()?).into()),
                    4 => SerialType::Integer(i32::from_be_bytes(data[cp..cp_end].try_into()?).into()),
                    8 => SerialType::Integer(i64::from_be_bytes(data[cp..cp_end].try_into()?).into()),
                    _ => unreachable!(),
                };
            } else {
                size_of_column = 0;
                st = SerialType::NULL;
            };

            columns.push(st);
            serial_type_pointer += bytes_read;
            column_pointer += size_of_column;
        }

        Ok(Self { columns })
    }

    pub fn get_column(&self, index: usize) -> &SerialType {
        &self.columns[index]
    }
}