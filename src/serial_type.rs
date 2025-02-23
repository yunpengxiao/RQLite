use std::fmt::Display;

/*
    Type            Size	    Meaning
    0	            0	        Value is a NULL.
    1	            1	        Value is an 8-bit twos-complement integer.
    2	            2	        Value is a big-endian 16-bit twos-complement integer.
    3	            3	        Value is a big-endian 24-bit twos-complement integer.
    4	            4	        Value is a big-endian 32-bit twos-complement integer.
    5	            6	        Value is a big-endian 48-bit twos-complement integer.
    6	            8	        Value is a big-endian 64-bit twos-complement integer.
    7	            8	        Value is a big-endian IEEE 754-2008 64-bit floating point number.
    8	            0	        Value is the integer 0. (Only available for schema format 4 and higher.)
    9	            0	        Value is the integer 1. (Only available for schema format 4 and higher.)
    10,11           variable	Reserved for internal use. These serial type codes will never appear in a well-formed database file, but they might be used in transient and temporary database files that SQLite sometimes generates for its own use. The meanings of these codes can shift from one release of SQLite to the next.
    N≥12 and even	(N-12)/2	Value is a BLOB that is (N-12)/2 bytes in length.
    N≥13 and odd	(N-13)/2	Value is a string in the text encoding and (N-13)/2 bytes in length. The nul terminator is not stored.
*/
#[derive(Debug, Clone)]
pub enum SerialType {
    Null,
    I8,
    I16,
    I24,
    I32,
    I48,
    I64,
    Float,
    Zero,
    One,
    String,
    Blob,
}

/*impl Display for SerialType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            SerialType::String(s) => write!(f, "{s}"),
            SerialType::Blob(b) => write!(f, "{b:?}"),
            SerialType::Integer(i) => write!(f, "{i:?}"),
            SerialType::NULL => write!(f, "NULL"),
        }
    }
}

// TryFrom definetion macro
macro_rules! convert {
    ($t:ty, $x:ident) => {
        #[automatically_derived]
        impl TryFrom<&SerialType> for $t {
            type Error = &'static str;

            fn try_from(st: &SerialType) -> std::result::Result<Self, Self::Error> {
                if let SerialType::$x(i) = st {
                    Ok(i.clone())
                } else {
                    Err("wrong")
                }
            }
        }
    };
}*/

//convert!(i64, Integer);
//convert!(Vec<u8>, Blob);
//convert!(String, String);

// impl TryFrom<&SerialType> for i64 {
//     type Error = &'static str;

//     fn try_from(st: &SerialType) -> std::result::Result<Self, Self::Error> {
//         if let SerialType::Integer(i) = st {
//             Ok(*i)
//         } else {
//             Err("wrong")
//         }
//     }
// }

/*impl TryInto<i64> for &SerialType {
    type Error = &'static str;

    fn try_into(self) -> std::result::Result<i64, Self::Error>  {
        match self {
            SerialType::Integer(i) => Ok(i),
            _ => Err("Not the integer type"),
        }
    }
}*/
