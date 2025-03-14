use std::fs::File;
use std::io::prelude::*;
use thiserror::Error;

use crate::cell::Cell;
use crate::utils;

// You need to set RUST_LIB_BACKTRACE=1 to enable backtrace here.
// Running the code like "RUST_LIB_BACKTRACE=1 cargo run -- sample.db tables"
#[derive(Debug, Error)]
pub enum MyError {
    #[error("Io Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Offset Error: {0}")]
    Offset(#[from] std::num::TryFromIntError),

    #[error("utf8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Slice error: {0}")]
    Slice(#[from] std::array::TryFromSliceError),
}

pub type Result<T> = core::result::Result<T, MyError>;

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
    pub page_num: u64,
    pub cells: Vec<Cell>,
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
    pub rightmost_pointer: Option<u32>,
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

// The page_num here starts with 0
impl TableLeafPage {
    pub fn from(buffer: &[u8], page_num: u64, page_size: u64) -> Self {
        let page_header = PageHeader::from(buffer, page_num).unwrap();
        let cells = Self::get_cells_from(
            buffer,
            page_num,
            page_header.cell_count as usize,
            page_header.get_header_size(),
        );
        Self {
            page_header,
            page_num,
            cells,
        }
    }

    pub fn table_count(&self) -> u16 {
        self.page_header.cell_count
    }

    fn get_cells_from(
        buffer: &[u8],
        page_num: u64,
        cell_count: usize,
        header_size: usize,
    ) -> Vec<Cell> {
        let mut cells: Vec<Cell> = Vec::new();
        let cell_pointers = &buffer[header_size..header_size + cell_count * 2];
        for arr in cell_pointers.chunks_exact(2) {
            let offset = if page_num == 0 {
                u16::from_be_bytes(arr.try_into().unwrap()) - FileHeader::FILE_HEADER_SIZE as u16
            } else {
                u16::from_be_bytes(arr.try_into().unwrap())
            };
            cells.push(Cell::from(&buffer[offset as usize..]).unwrap());
        }
        cells
    }
}

impl PageHeader {
    pub fn from(buffer: &[u8], page_num: u64) -> Result<Self> {
        // This offset is the offset since the beginning of page not the buffer.
        let cell_content_offset = if page_num == 0 {
            u32::from_be_bytes([buffer[5], buffer[6], buffer[7], buffer[8]])
                - FileHeader::FILE_HEADER_SIZE as u32
        } else {
            u32::from_be_bytes([buffer[5], buffer[6], buffer[7], buffer[8]])
        };
        Ok(Self {
            page_type: utils::get_page_type(buffer[0]),
            first_freeblock: u16::from_be_bytes([buffer[1], buffer[2]]),
            cell_count: u16::from_be_bytes([buffer[3], buffer[4]]),
            cell_content_offset,
            fragmented_bytes_count: buffer[9],
            rightmost_pointer: None,
        })
    }

    pub fn get_header_size(&self) -> usize {
        if self.rightmost_pointer.is_some() {
            12
        } else {
            8
        }
    }
}
