use crate::page::FileHeader;
use crate::page_scanner::PageScanner;
use std::fs::File;

#[derive(Debug)]
pub struct Database {
    pub file_header: FileHeader,
    // This member actually need to be thread safe mutable
    pub page_scanner: PageScanner,
}

impl Database {
    pub fn from(db_path: String) -> Self {
        let mut db_file = File::open(db_path).unwrap();
        let file_header = FileHeader::from(&mut db_file).unwrap();
        let page_size = file_header.page_size as u64;
        Self {
            file_header,
            page_scanner: PageScanner::from(db_file, 1, page_size),
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
        while let Some(record) = self.page_scanner.get_next_record() {
            result.push(record.columns[1].value());
        }
        result
    }
}
