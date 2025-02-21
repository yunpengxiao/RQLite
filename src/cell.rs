use crate::page::Result;
use crate::record::Record;

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
