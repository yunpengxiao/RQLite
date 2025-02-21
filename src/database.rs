use core::panic;
use std::fs::File;
use std::io::SeekFrom;
use std::io::prelude::*;

use crate::page::TableLeafPage;
use crate::page::{FileHeader, Page};
use crate::utils::get_page_type;

#[derive(Debug)]
pub struct Database {
    pub file_header: FileHeader,
    pub pages: Vec<Page>,
}

impl Database {
    pub fn from(file: &mut File) -> Self {
        let file_header = FileHeader::from(file).unwrap();
        let mut pages: Vec<Page> = Vec::new();
        for i in 0..file_header.page_count {
            let buffer: &Vec<u8> =
                &Self::load_raw_page(file, i as u64, file_header.page_size as u64);
            let page = match get_page_type(buffer[0]) {
                crate::page::PageType::TableLeaf => Page::TableLeaf(TableLeafPage::from(
                    buffer,
                    i as u64,
                    file_header.page_size as u64,
                )),
                _ => panic!("wrong"),
            };
            pages.push(page);
        }

        /*let first_page = &pages[0];
        let mut table_schemas: HashMap<String, TableSchema> = HashMap::new();
        for n in 0..first_page.row_reader.pointers.len() {
            let table_name = first_page.row_reader.read(n as u32)[1].to_string();
            let table_sql = first_page.row_reader.read(n as u32)[4].to_string();
            table_schemas.insert(table_name, TableSchema::from(&table_sql));
        }*/

        Self { file_header, pages }
    }

    pub fn get_page_size(&self) -> u16 {
        self.file_header.page_size
    }

    pub fn get_page_count(&self) -> u32 {
        self.file_header.page_count
    }

    fn load_raw_page(file: &mut File, page_num: u64, page_size: u64) -> Vec<u8> {
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
        file.seek(SeekFrom::Start(offset)).unwrap();
        file.read_exact(&mut buffer).unwrap(); // buffer is coverted to &[u8] because vec implements AsRef<T>
        buffer
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

    /*pub fn get_table_names(&self) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        match &self.pages[0] {
            Page::TableLeaf(first_page) => {
                for n in 0..first_page.row_reader.cells.len() {
                    result.push(first_page.row_reader.read(n as u32)[1].to_string());
                }
                result
            }
            _ => panic!("Something wronge"),
        }
    }*/

    /*pub fn count_rows(&self, table_name: &str) -> usize {
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
                    println!("the result is {:?}", row);
                    match row[i] {
                        SerialType::String(s) => result.push(s.clone()),
                        _ => println!("the format is not correct"),
                    }
                }
            }
        }
        result
    }*/

    /*fn get_table_location(&self, table_name: &str) -> usize {
        let first_page = &self.pages[0];
        // How to return the value directly in the for loop here?
        for n in 0..first_page.row_reader.pointers.len() {
            let name = first_page.row_reader.read(n as u32)[1].to_string();
            if name == table_name {
                //let table_location: i64 = self.pages[0].row_reader.read(n as u32)[3].try_into().unwrap();
                //println!("value: {}", table_location);
                match first_page.row_reader.read(n as u32)[3] {
                    SerialType::Integer(i) => {
                        println!("The location for table {} is {}", table_name, i);
                        return *i as usize;
                    }
                    _ => {
                        println!("Somthing is wrong!");
                    }
                }
            }
        }
        0
    }*/
}
