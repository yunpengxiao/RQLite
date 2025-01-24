use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use thiserror::Error;

use crate::parser::sql_query;
use crate::parser::SqlStatement;
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

    /*#[error("Parser error: {0}")]
    Parser(#[from] IResult<&'a str, SqlStatement>, std::backtrace::Backtrace),
    */
}

type Result<T> = core::result::Result<T, MyError>;

#[derive(Debug, Clone)]
pub struct Database {
    pub file_header: FileHeader,
    pub pages: Vec<PageReader>,
    pub table_schemas: HashMap<String, TableSchema>,
}

impl Database {
    pub fn from(file: &mut File) -> Result<Self> {
        let file_header = FileHeader::from(file)?;
        let mut pages: Vec<PageReader> = Vec::new();
        for i in 1..=file_header.page_count {
            let page = PageReader::from(file, i as u64, file_header.page_size as u64);
            pages.push(page);
        }

        let first_page = &pages[0];
        let mut table_schemas: HashMap<String, TableSchema> = HashMap::new();
        for n in 0..first_page.row_reader.pointers.len() {
            let table_name = first_page.row_reader.read(n as u32)[1].to_string();
            let table_sql = first_page.row_reader.read(n as u32)[4].to_string();
            table_schemas.insert(table_name, TableSchema::from(&table_sql));
        } 

        Ok(Self {
            file_header,
            pages,
            table_schemas,
        })
    }

    pub fn get_page_size(&self) -> u16 {
        self.file_header.page_size
    }

    pub fn get_page_count(&self) -> u32 {
        self.file_header.page_count
    }

    /*
        Every SQLite database contains a single "schema table" that stores the schema for that database. 
        The schema for a database is a description of all of the other tables, indexes, triggers, and 
        views that are contained within the database. The schema table looks like this:

            CREATE TABLE sqlite_schema(
            type text,
            name text,
            tbl_name text,
            rootpage integer,
            sql text
            );

        we can parse this table to get all the table name and schema. This table is in the first page of
        every Sqlite database file.
    */ 
    
    pub fn get_table_names(&self) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        let first_page = &self.pages[0];
        for n in 0..first_page.row_reader.pointers.len() {
            result.push(first_page.row_reader.read(n as u32)[1].to_string());
        } 
        result
    }

    pub fn count_rows(&self, table_name: &str) -> usize {
        let root_num = self.get_table_location(table_name);
        let page = &self.pages[root_num - 1];
        return page.row_reader.cells.len();
    }

    pub fn get_column(&self, table_name: &str, column_name: &str) -> Vec<String> {
        let mut result = Vec::new();
        let table_schema = self.table_schemas.get(table_name).unwrap();
        for i in 0..table_schema.cols.len() {
            if table_schema.cols[i] == column_name {
                let table_location = self.get_table_location(table_name);
                let page = &self.pages[table_location - 1];
                for n in 0..page.row_reader.cells.len() {
                    let row = page.row_reader.read(n as u32);
                    match row[i] {
                        SerialType::String(s) => result.push(s.clone()),
                        _ => println!("the format is not correct"),
                    }
                }
            }
        }
        result
    }

    fn get_table_location(&self, table_name: &str) -> usize {
        let first_page = &self.pages[0];
        // How to return the value directly in the for loop here?
        for n in 0..first_page.row_reader.pointers.len() {
            let name = first_page.row_reader
                .read(n as u32)[1].to_string();
            if name == table_name {
                /*let table_location: i64 = first_page
                    .row_reader.read(n as u32)[3].try_into().unwrap();*/
                match first_page.row_reader.read(n as u32)[3] {
                    SerialType::Integer(i) => {
                        println!("The location for table {} is {}", 
                        table_name, i);
                        return *i as usize;
                    }
                    _ => {
                        println!("Somthing is wrong!");
                    }
                }
            }
        }
        0
    }
}



/*
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

impl FileHeader {
const FILE_HEADER_SIZE: usize = 100;

    pub fn from(file: &mut File) -> Result<Self> {
        let mut header = [0; Self::FILE_HEADER_SIZE];
        file.read_exact(&mut header)?;
        Ok(Self {
            page_size: u16::from_be_bytes([header[16], header[17]]),
            page_count: u32::from_be_bytes([header[28], header[29], header[30], header[31]]),
        })
    }
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
pub struct PageReader {
    pub page_header: PageHeader,
    pub row_reader: RowReader,
    pub page_num: u64,
}

impl PageReader {
    pub fn from (file: &mut File, page_num: u64, page_size: u64) -> Self {
        Self {
            page_header: PageHeader::from(file, page_num, page_size).unwrap(),
            row_reader: RowReader::from(file, page_num, page_size).unwrap(),
            page_num,
        }
    }

    pub fn table_count(&self) -> u16 {
        self.page_header.cell_count
    }
}

/*
    Page Header Layout
        Offset	Size	Description
        0	1	The one-byte flag at offset 0 indicating the b-tree page type.
                A value of 2 (0x02) means the page is an interior index b-tree page.
                A value of 5 (0x05) means the page is an interior table b-tree page.
                A value of 10 (0x0a) means the page is a leaf index b-tree page.
                A value of 13 (0x0d) means the page is a leaf table b-tree page.
                Any other value for the b-tree page type is an error.
        1	2	The two-byte integer at offset 1 gives the start of the first freeblock on the page, or is zero if there are no freeblocks.
        3	2	The two-byte integer at offset 3 gives the number of cells on the page.
        5	2	The two-byte integer at offset 5 designates the start of the cell content area. A zero value for this integer is interpreted as 65536.
        7	1	The one-byte integer at offset 7 gives the number of fragmented free bytes within the cell content area.
        8	4	The four-byte page number at offset 8 is the right-most pointer. This value appears in the header of interior b-tree pages only and is omitted from all other pages.
*/

#[derive(Debug, Clone)]
pub struct PageHeader {
    pub cell_count: u16,
}

impl PageHeader {
const MAX_PAGE_HEADER_SIZE: usize = 8;

    pub fn from(file: &mut File, page_num: u64, page_size: u64) -> Result<Self> {
        let mut buffer = [0; 2];
        let offset = get_page_start_offset(page_num, page_size);
        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(&mut buffer)?;
        Ok(Self {
            cell_count: u16::from_be_bytes([buffer[0], buffer[1]]),
        })
    }
}

#[derive(Debug, Clone)]
struct RowReader {
    pub pointers: Vec<u16>,
    pub cells: Vec<Cell>,
}

impl RowReader {
    const BUFFER_SIZE: usize = 40960;

    pub fn from(file: &mut File, page_num: u64, page_size: u64) -> Result<Self> {
        let cell_count = Self::get_cell_count(file, page_num, page_size).unwrap();
        let mut buffer = vec![0; (cell_count as usize) * 2];
        let mut cell_pointers = Vec::new();
        let page_offset = get_page_start_offset(page_num, page_size);
        // Page header size can be 12 bytes too, just use 8 here for simplicity
        file.seek(SeekFrom::Start(page_offset + PageHeader::MAX_PAGE_HEADER_SIZE as u64))?;
        file.read_exact(&mut buffer[..])?;
        for arr in buffer.as_slice().as_array_iter() {
            let offset = u16::from_be_bytes(*arr);
            cell_pointers.push(offset);
        }

        let mut cells: Vec<Cell> = Vec::new();
        for cell_location in &cell_pointers {
            let mut buffer = [0; Self::BUFFER_SIZE];
            file.seek(SeekFrom::Start(page_offset + (*cell_location) as u64))?;
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
        file.seek(SeekFrom::Start(get_page_start_offset(page_num, page_size)+ 3))?;
        file.read_exact(&mut buffer)?;
        Ok(u16::from_be_bytes([buffer[0], buffer[1]]))
    }

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

impl TryInto<i64> for SerialType {
    type Error = &'static str;

    fn try_into(self) -> std::result::Result<i64, Self::Error>  {
        match self {
            SerialType::Integer(i) => Ok(i),
            _ => Err("Not the integer type"),
        }
    }
}

#[derive(Debug, Clone)]
struct TableSchema {
    pub table_name: String,
    pub cols: Vec<String>,
}

impl TableSchema {
    pub fn from(sql: &String) -> Self {
        let (_, sql_cmd) = sql_query(sql).unwrap();
        let mut table_name: String = String::new();
        let mut cols: Vec<String> = Vec::new();
        match sql_cmd {
            SqlStatement::CREATE(cs) => {
                table_name = cs.table_name;
                cols = cs.cols;
            }
            _ => println!("Something is wrong, the schema is not a creation sql."),
        }
        Self {
            table_name,
            cols,
        }
    }
}

fn get_page_start_offset(page_num: u64, page_size: u64) -> u64 {
    let start = (page_num - 1) * page_size;
    if page_num == 1 {
        start + FileHeader::FILE_HEADER_SIZE as u64
    } else {
        start
    }
}