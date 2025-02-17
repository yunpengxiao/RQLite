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
    Utf8(
        #[from] std::string::FromUtf8Error,
        std::backtrace::Backtrace,
    ),

    #[error("Slice error: {0}")]
    Slice(
        #[from] std::array::TryFromSliceError,
        std::backtrace::Backtrace,
    ),
}

type Result<T> = core::result::Result<T, MyError>;

/*
   File header only exists in the first page.
   File Header Format
       Offset	Size	Description
       0	    16	    The header string: "SQLite format 3\000"
       16	    2	    The database page size in bytes. Must be a power of two between 512 and 32768 inclusive, or the value 1 representing a page size of 65536.
       18	    1	    File format write version. 1 for legacy; 2 for WAL.
       19	    1	    File format read version. 1 for legacy; 2 for WAL.
       20	    1	    Bytes of unused "reserved" space at the end of each page. Usually 0.
       21	    1	    Maximum embedded payload fraction. Must be 64.
       22	    1	    Minimum embedded payload fraction. Must be 32.
       23	    1	    Leaf payload fraction. Must be 32.
       24	    4	    File change counter.
       28	    4	    Size of the database file in pages. The "in-header database size".
       32	    4	    Page number of the first freelist trunk page.
       36	    4	    Total number of freelist pages.
       40	    4	    The schema cookie.
       44	    4	    The schema format number. Supported schema formats are 1, 2, 3, and 4.
       48	    4	    Default page cache size.
       52	    4	    The page number of the largest root b-tree page when in auto-vacuum or incremental-vacuum modes, or zero otherwise.
       56	    4	    The database text encoding. A value of 1 means UTF-8. A value of 2 means UTF-16le. A value of 3 means UTF-16be.
       60	    4	    The "user version" as read and set by the user_version pragma.
       64	    4	    True (non-zero) for incremental-vacuum mode. False (zero) otherwise.
       68	    4	    The "Application ID" set by PRAGMA application_id.
       72	    20	    Reserved for expansion. Must be zero.
       92	    4	    The version-valid-for number.
       96	    4	    SQLITE_VERSION_NUMBER
*/

#[derive(Debug, Clone)]
pub struct FileHeader {
    pub page_size: u16,
    pub page_count: u32,
}

/* And there are 4 types of page, the type of the page is included at the begining of page header:
    A value of 2 (0x02) means the page is an interior index b-tree page.
    A value of 5 (0x05) means the page is an interior table b-tree page.
    A value of 10 (0x0a) means the page is a leaf index b-tree page.
    A value of 13 (0x0d) means the page is a leaf table b-tree page.
*/
#[derive(Debug)]
pub enum Page {
    TableLeaf(TableLeafPage),
}

#[derive(Debug, Copy, Clone)]
pub enum PageType {
    TableLeaf,
    IndexLeaf,
    TableInterior,
    IndexInterior,
}

/*
    A b-tree page is divided into regions in the following order:
        * The 100-byte database file header (found on page 1 only)
        * The 8 or 12 byte b-tree page header
        * The cell pointer array
        * Unallocated space
        * The cell content area
        * The reserved region
*/
#[derive(Debug, Clone)]
pub struct TableLeafPage {
    pub page_header: PageHeader,
    pub row_reader: RowReader,
    pub page_num: u64,
}

/*
    Page Header Layout
    Offset	Size	Description
        0	1	The one-byte flag at offset 0 indicating the b-tree page type.
        1	2	2-byte integer representing the offset of the first free block in the page, or zero if there is no freeblock.
        3	2	2-byte integer representing the number of cells in the page.
        5	2	2-byte integer representing the offset of the first cell.
        7	1	The one-byte integer at offset 7 gives the number of fragmented free bytes within the cell content area.
        8	4	The four-byte page number at offset 8 is the right-most pointer. This value appears in the header of interior b-tree pages only and is omitted from all other pages.
*/
#[derive(Debug, Clone)]
pub struct PageHeader {
    pub page_type: PageType,
    pub first_freeblock: u16,
    pub cell_count: u16,
    pub cell_content_offset: u32,
    pub fragmented_bytes_count: u8,
}

#[derive(Debug, Clone)]
struct RowReader {
    pub pointers: Vec<u16>,
    pub cells: Vec<Cell>,
}

/* Cell Format:
 * The size of the record, in bytes (varint)
 * The rowid (varint)
 * The record (record format)
 */
#[derive(Debug, Clone)]
struct Cell {
    pub size_of_record: usize,
    pub rowid: i64,
    pub record: Record,
}

/*
   Record Format
     * Header:
      - Size of the header, including this value (varint)
      - Serial type code for each column in the record, in order (varint)
     * Body:
      - The value of each column in the record, in order (format varies based on serial type code)
*/
#[derive(Debug, Clone)]
struct Record {
    pub columns: Vec<SerialType>,
}

impl FileHeader {
    pub const FILE_HEADER_SIZE: usize = 100;

    pub fn from(file: &mut File) -> Result<Self> {
        let mut header = [0; Self::FILE_HEADER_SIZE];
        file.read_exact(&mut header)?;
        Ok(Self {
            page_size: u16::from_be_bytes([header[16], header[17]]),
            page_count: u32::from_be_bytes([header[28], header[29], header[30], header[31]]),
        })
    }
}

impl TableLeafPage {
    pub fn from(buffer: &[u8], page_num: u64, page_size: u64) -> Self {
        Self {
            page_header: PageHeader::from(buffer, page_num, page_size).unwrap(),
            row_reader: RowReader::from(buffer, page_num, page_size).unwrap(),
            page_num,
        }
    }

    pub fn table_count(&self) -> u16 {
        self.page_header.cell_count
    }
}

impl PageHeader {
    const MAX_PAGE_HEADER_SIZE: usize = 8;

    pub fn from(buffer: &[u8], page_num: u64, page_size: u64) -> Result<Self> {
        Ok(Self {
            cell_count: u16::from_be_bytes([buffer[0], buffer[1]]),
        })
    }
}

impl RowReader {
    const BUFFER_SIZE: usize = 4096;

    pub fn from(file: &mut File, page_num: u64, page_size: u64) -> Result<Self> {
        let cell_count = Self::get_cell_count(file, page_num, page_size).unwrap();
        //println!("Page {} We have {} cells", page_num, cell_count);
        let mut buffer = vec![0; (cell_count as usize) * 2];
        let mut cell_pointers = Vec::new();
        let page_offset =
            get_page_start_offset(page_num, page_size) + PageHeader::MAX_PAGE_HEADER_SIZE as u64;
        // Page header size can be 12 bytes too, just use 8 here for simplicity
        //println!("Reading cell pointers from offset {} with bytes {}", page_offset,  (cell_count as usize) * 2);
        file.seek(SeekFrom::Start(page_offset))?;
        file.read_exact(&mut buffer[..])?;
        for arr in buffer.as_slice().as_array_iter() {
            let offset = u16::from_be_bytes(*arr);
            cell_pointers.push(offset);
        }

        let mut cells: Vec<Cell> = Vec::new();
        for cell_location in &cell_pointers {
            let mut buffer = [0; Self::BUFFER_SIZE];
            let offset = (page_num - 1) * page_size + (*cell_location) as u64;
            /*println!(
                "Reading cells from offset {} with size {}.",
                offset,
                Self::BUFFER_SIZE
            );*/
            file.seek(SeekFrom::Start(offset))?;
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

    fn get_cell_count(file: &mut File, page_num: u64, page_size: u64) -> Result<u16> {
        let mut buffer = [0; 2];
        let offset = get_page_start_offset(page_num, page_size) + 3;
        //println!("Getting cell count from {} with size 2.", offset);
        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(&mut buffer)?;
        Ok(u16::from_be_bytes([buffer[0], buffer[1]]))
    }
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
                st = SerialType::Blob(
                    data[(column_pointer as usize)
                        ..(column_pointer as usize) + (size_of_column as usize)]
                        .to_vec(),
                );
            } else if serial_type >= 13 && serial_type % 2 != 0 {
                size_of_column = (serial_type - 13) / 2;
                st = SerialType::String(String::from_utf8(
                    data[(column_pointer as usize)
                        ..(column_pointer as usize) + (size_of_column as usize)]
                        .to_vec(),
                )?);
            } else if serial_type >= 1 && serial_type <= 4 {
                size_of_column = serial_type;
                let cp = column_pointer as usize;
                let cp_end = cp + size_of_column as usize;
                st = match serial_type {
                    1 => {
                        SerialType::Integer(i8::from_be_bytes(data[cp..cp_end].try_into()?).into())
                    }
                    2 => {
                        SerialType::Integer(i16::from_be_bytes(data[cp..cp_end].try_into()?).into())
                    }
                    4 => {
                        SerialType::Integer(i32::from_be_bytes(data[cp..cp_end].try_into()?).into())
                    }
                    8 => {
                        SerialType::Integer(i64::from_be_bytes(data[cp..cp_end].try_into()?).into())
                    }
                    _ => unreachable!(),
                };
            } else {
                size_of_column = 0;
                st = SerialType::NULL;
            };
            //println!("Read col {:?} with size {}", st, size_of_column);

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

/*
    Type            Size	    Meaning
    0	            0	        Value is a NULL.
    1	            1	        Value is an 8-bit twos-complement integer.
    2	            2	        Value is a big-endian 16-bit twos-complement integer.
    3	            3	        Value is a big-endian 24-bit twos-complement integer.
    4	            4	        Value is a big-endian 32-bit twos-complement integer.
    5	            6	        Value is a big-endian 48-bit twos-complement integer.
    6	            8	        Value is a big-endian 64-bit twos-complement integer.
    7	            8	        Value is a big-endian IEEE 754-2008 64-bit floating point number.
    8	            0	        Value is the integer 0. (Only available for schema format 4 and higher.)
    9	            0	        Value is the integer 1. (Only available for schema format 4 and higher.)
    10,11           variable	Reserved for internal use. These serial type codes will never appear in a well-formed database file, but they might be used in transient and temporary database files that SQLite sometimes generates for its own use. The meanings of these codes can shift from one release of SQLite to the next.
    N≥12 and even	(N-12)/2	Value is a BLOB that is (N-12)/2 bytes in length.
    N≥13 and odd	(N-13)/2	Value is a string in the text encoding and (N-13)/2 bytes in length. The nul terminator is not stored.
*/
#[derive(Debug, Clone)]
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

// TryFrom definetion macro
macro_rules! convert {
    ($t:ty, $x:ident) => {
        #[automatically_derived]
        impl TryFrom<&SerialType> for $t {
            type Error = &'static str;

            fn try_from(st: &SerialType) -> std::result::Result<Self, Self::Error> {
                if let SerialType::$x(i) = st {
                    Ok(i.clone())
                } else {
                    Err("wrong")
                }
            }
        }
    };
}

convert!(i64, Integer);
convert!(Vec<u8>, Blob);
convert!(String, String);

// impl TryFrom<&SerialType> for i64 {
//     type Error = &'static str;

//     fn try_from(st: &SerialType) -> std::result::Result<Self, Self::Error> {
//         if let SerialType::Integer(i) = st {
//             Ok(*i)
//         } else {
//             Err("wrong")
//         }
//     }
// }

/*impl TryInto<i64> for &SerialType {
    type Error = &'static str;

    fn try_into(self) -> std::result::Result<i64, Self::Error>  {
        match self {
            SerialType::Integer(i) => Ok(i),
            _ => Err("Not the integer type"),
        }
    }
}*/

fn get_page_start_offset(page_num: u64, page_size: u64) -> u64 {
    let start = (page_num - 1) * page_size;
    if page_num == 1 {
        start + FileHeader::FILE_HEADER_SIZE as u64
    } else {
        start
    }
}
