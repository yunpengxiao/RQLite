use core::panic;
use std::collections::HashMap;
use std::fs::File;
use std::io::SeekFrom;
use std::io::prelude::*;
use std::path::PathBuf;

use crate::page::TableLeafPage;
use crate::page::{FileHeader, Page};
use crate::utils::get_page_type;

#[derive(Debug)]
pub struct Database {
    pub file_header: FileHeader,
    pub db_file: File,
    // This member actually need to be thread safe mutable
    pub pages: HashMap<usize, Page>,
}

impl Database {
    pub fn from(db_path: String) -> Self {
        let mut db_file = File::open(db_path).unwrap();
        let file_header = FileHeader::from(&mut db_file).unwrap();
        let pages = HashMap::new();

        Self {
            file_header,
            db_file,
            pages,
        }
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

    pub fn get_table_names(&mut self) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        let data = self.load_raw_page(1, self.get_page_size() as u64);
        let first_page = TableLeafPage::from(&data, 0, self.get_page_size() as u64);
        self.pages
            .entry(1)
            .or_insert(Page::TableLeaf(first_page.clone()));
        for cell in first_page.cells {
            result.push(cell.record.columns[1].value());
        }
        result
    }

    fn load_raw_page(&mut self, page_num: u64, page_size: u64) -> Vec<u8> {
        let page_num = page_num - 1;
        let raw_page_size = if page_num == 0 {
            page_size - FileHeader::FILE_HEADER_SIZE as u64
        } else {
            page_size
        };
        let mut buffer = vec![0; raw_page_size as usize];
        let offset = if page_num == 0 {
            FileHeader::FILE_HEADER_SIZE as u64
        } else {
            page_num * page_size
        };
        println!(
            "Reading from offset {} with {} bytes.",
            offset, raw_page_size
        );
        self.db_file.seek(SeekFrom::Start(offset)).unwrap();
        self.db_file.read_exact(&mut buffer).unwrap(); // buffer is coverted to &[u8] because vec implements AsRef<T>
        buffer
    }
}
