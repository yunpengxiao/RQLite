use core::str;

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
        let mut column_pointer = record_head_size as usize;
        let mut serial_type_pointer: usize = first_type_offset;
        while serial_type_pointer != record_head_size as usize {
            let (serial_type, bytes_read) = read_variant(&data[serial_type_pointer..]);
            let mut size_of_column = 0 as usize;
            let st: SerialType;
            if serial_type >= 12 && serial_type % 2 == 0 {
                size_of_column = (serial_type as usize - 12) / 2;
                st = SerialType::Blob(Box::from(
                    &data[column_pointer..column_pointer + size_of_column],
                ));
            } else if serial_type >= 13 && serial_type % 2 != 0 {
                size_of_column = (serial_type as usize - 13) / 2;
                let strv = str::from_utf8(&data[column_pointer..column_pointer + size_of_column]);
                st = SerialType::String(String::from(strv.unwrap()));
            } else {
                st = match serial_type {
                    0 => SerialType::Null,
                    1 => {
                        size_of_column = 1;
                        SerialType::I8(data[column_pointer] as i8)
                    }
                    2 => {
                        size_of_column = 2;
                        SerialType::I16(i16::from_be_bytes(
                            data[column_pointer..column_pointer + size_of_column]
                                .try_into()
                                .unwrap(),
                        ))
                    }
                    3 => {
                        size_of_column = 3;
                        SerialType::I24(i32::from_be_bytes(
                            data[column_pointer..column_pointer + size_of_column]
                                .try_into()
                                .unwrap(),
                        ))
                    }
                    4 => {
                        size_of_column = 4;
                        SerialType::I32(i32::from_be_bytes(
                            data[column_pointer..column_pointer + size_of_column]
                                .try_into()
                                .unwrap(),
                        ))
                    }
                    5 => {
                        size_of_column = 6;
                        SerialType::I48(i64::from_be_bytes(
                            data[column_pointer..column_pointer + size_of_column]
                                .try_into()
                                .unwrap(),
                        ))
                    }
                    6 => {
                        size_of_column = 8;
                        SerialType::I64(i64::from_be_bytes(
                            data[column_pointer..column_pointer + size_of_column]
                                .try_into()
                                .unwrap(),
                        ))
                    }
                    7 => {
                        size_of_column = 8;
                        SerialType::Float(f64::from_be_bytes(
                            data[column_pointer..column_pointer + size_of_column]
                                .try_into()
                                .unwrap(),
                        ))
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
