//! Minimal Bond CompactBinary v1 codec, just enough to read/write the Night Light
//! registry blobs. Adapted from <https://github.com/kvnxiao/win-nightlight-cli>.

#[allow(unused)]
mod types;
#[allow(unused)]
mod varint;

#[allow(unused)]
pub mod reader;
#[allow(unused)]
pub mod value;
#[allow(unused)]
pub mod writer;

pub use reader::{CompactBinaryReader, FieldHeader};
pub use types::BondType;
#[allow(unused_imports)]
pub use value::{BondStruct, BondValue};
pub use writer::CompactBinaryWriter;

#[derive(Debug)]
pub enum BondError {
    UnexpectedEof(usize),
    InvalidHeader,
    InvalidTypeId(u8),
    VarintOverflow,
    InvalidUtf8,
    InvalidUtf16,
    MissingField(u16),
    UnexpectedFieldType(u16),
}

impl std::fmt::Display for BondError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BondError::UnexpectedEof(pos) => write!(f, "Unexpected end of data at position {pos}"),
            BondError::InvalidHeader => write!(f, "Invalid marshaled header"),
            BondError::InvalidTypeId(id) => write!(f, "Invalid type ID: {id}"),
            BondError::VarintOverflow => write!(f, "Varint overflow"),
            BondError::InvalidUtf8 => write!(f, "Invalid UTF-8 string"),
            BondError::InvalidUtf16 => write!(f, "Invalid UTF-16 string"),
            BondError::MissingField(id) => write!(f, "Missing required field {id}"),
            BondError::UnexpectedFieldType(id) => write!(f, "Unexpected field type for field {id}"),
        }
    }
}

impl std::error::Error for BondError {}
