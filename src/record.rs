use crate::serial_type::SerialType;

use crate::page::Result;
use crate::utils::read_variant;

/*
   Record Format
     * Header:
      - Size of the header, including this value (varint)
      - Serial type code for each column in the record, in order (varint)
     * Body:
      - The value of each column in the record, in order (format varies based on serial type code)
*/
#[derive(Debug, Clone)]
pub struct Record {
    pub columns: Vec<Column>,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub offset: usize,
    pub serial_type: SerialType,
}

impl Record {
    pub fn from(data: &[u8]) -> Result<Self> {
        let mut columns = Vec::new();
        let (record_head_size, first_type_offset) = read_variant(&data[..]);
        let mut column_pointer = record_head_size;
        let mut serial_type_pointer: usize = first_type_offset;
        while serial_type_pointer != record_head_size as usize {
            let (serial_type, bytes_read) = read_variant(&data[serial_type_pointer..]);
            let mut size_of_column = 0;
            let st: SerialType;
            if serial_type >= 12 && serial_type % 2 == 0 {
                size_of_column = (serial_type - 12) / 2;
                st = SerialType::Blob;
            } else if serial_type >= 13 && serial_type % 2 != 0 {
                size_of_column = (serial_type - 13) / 2;
                st = SerialType::String;
            } else {
                st = match serial_type {
                    0 => SerialType::Null,
                    1 => {
                        size_of_column = 1;
                        SerialType::I8
                    }
                    2 => {
                        size_of_column = 2;
                        SerialType::I16
                    }
                    3 => {
                        size_of_column = 3;
                        SerialType::I24
                    }
                    4 => {
                        size_of_column = 4;
                        SerialType::I32
                    }
                    5 => {
                        size_of_column = 6;
                        SerialType::I48
                    }
                    6 => {
                        size_of_column = 8;
                        SerialType::I64
                    }
                    7 => {
                        size_of_column = 8;
                        SerialType::Float
                    }
                    8 => SerialType::Zero,
                    9 => SerialType::One,
                    _ => panic!("invalid serial type {}", serial_type),
                };
            }
            let col = Column {
                offset: column_pointer as usize,
                serial_type: st,
            };

            columns.push(col);
            serial_type_pointer += bytes_read;
            column_pointer += size_of_column;
        }

        Ok(Self { columns })
    }
}

impl Column {
    pub fn value(&self) -> String {
        String::new()
    }
}
