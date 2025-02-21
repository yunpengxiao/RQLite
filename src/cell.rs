use crate::{serial_type::SerialType, utils::read_variant};
use crate::page::Result;

/* Cell Format:
 * The size of the record, in bytes (varint)
 * The rowid (varint)
 * The record (record format)
 */

#[derive(Debug, Clone)]
pub struct Cell {
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
