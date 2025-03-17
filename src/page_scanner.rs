use crate::page::{FileHeader, Page, TableLeafPage};
use crate::record::Record;
use std::fs::File;
use std::io::SeekFrom;
use std::io::prelude::*;

#[derive(Debug)]
pub struct PageScanner {
    db_file: File,
    start_page_num: u64,
    page_size: u64,
    current_position: PositionedPage,
}

impl PageScanner {
    pub fn from(db_file: File, page_num: u64, page_size: u64) -> Self {
        Self {
            db_file,
            start_page_num: page_num,
            page_size,
            current_position: PositionedPage {
                page_num,
                page: None,
                position: 0,
            },
        }
    }

    pub fn get_next_record(&mut self) -> Option<Record> {
        if self.current_position.page.is_none() {
            let raw_page_data = self.load_raw_page(self.start_page_num, self.page_size);
            let page = TableLeafPage::from(&raw_page_data, self.start_page_num == 1);
            self.current_position.page = Some(Page::TableLeaf(page));
        }
        self.current_position.next_record()
    }

    fn load_raw_page(&mut self, page_num: u64, page_size: u64) -> Vec<u8> {
        let page_num = page_num - 1;
        let raw_page_size = if page_num == 0 {
            self.db_file
                .seek(SeekFrom::Start(FileHeader::FILE_HEADER_SIZE as u64))
                .unwrap();
            page_size - FileHeader::FILE_HEADER_SIZE as u64
        } else {
            self.db_file
                .seek(SeekFrom::Start(page_num * page_size))
                .unwrap();
            page_size
        };

        let mut data = vec![0; raw_page_size as usize];
        self.db_file.read_exact(&mut data).unwrap();
        data
    }
}

#[derive(Debug)]
struct PositionedPage {
    page_num: u64,
    page: Option<Page>,
    position: u64,
}

impl PositionedPage {
    pub fn next_record(&mut self) -> Option<Record> {
        match &self.page {
            Some(Page::TableLeaf(page)) => {
                if self.position < page.cells.len() as u64 {
                    let record = &page.cells[self.position as usize].record;
                    self.position = self.position + 1;
                    Some(record.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
