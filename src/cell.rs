use crate::page::Result;
use crate::record::Record;

/* Table B-Tree Leaf Cell (header 0x0d):

A varint which is the total number of bytes of payload, including any overflow
A varint which is the integer key, a.k.a. "rowid"
The initial portion of the payload that does not spill to overflow pages.
A 4-byte big-endian integer page number for the first page of the overflow page list - omitted if all payload fits on the b-tree page.
*/

#[derive(Debug, Clone)]
pub struct Cell {
    pub size_of_record: usize,
    pub rowid: i64,
    pub record: Record,
}

/*Table B-Tree Interior Cell (header 0x05):

A 4-byte big-endian page number which is the left child pointer.
A varint which is the integer key
*/

#[derive(Debug, Clone)]
pub struct TableInteriorCell {
    pub left_child_page: u32,
    pub key: i64,
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

impl TableInteriorCell {
    pub fn from(data: &[u8]) -> Result<Self> {
        use crate::utils::read_variant;

        let left_child_page = u32::from_be_bytes(data[..4].try_into().unwrap());
        let (key, _) = read_variant(&data[4..]);

        Ok(Self {
            left_child_page,
            key,
        })
    }
}
