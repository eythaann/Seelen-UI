/// Bond CompactBinary protocol magic bytes (COMPACT_PROTOCOL = 0x4243, stored as uint16 LE).
pub const COMPACT_BINARY_MAGIC: [u8; 2] = [0x43, 0x42];

/// CompactBinary version 1 (uint16 LE).
pub const COMPACT_BINARY_V1: [u8; 2] = [0x01, 0x00];

/// Bond data type identifiers (5-bit, used in field headers and container headers).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BondType {
    Bool = 2,
    UInt8 = 3,
    UInt16 = 4,
    UInt32 = 5,
    UInt64 = 6,
    Float = 7,
    Double = 8,
    String = 9,
    Struct = 10,
    List = 11,
    Set = 12,
    Map = 13,
    Int8 = 14,
    Int16 = 15,
    Int32 = 16,
    Int64 = 17,
    WString = 18,
}

impl TryFrom<u8> for BondType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            2 => Ok(BondType::Bool),
            3 => Ok(BondType::UInt8),
            4 => Ok(BondType::UInt16),
            5 => Ok(BondType::UInt32),
            6 => Ok(BondType::UInt64),
            7 => Ok(BondType::Float),
            8 => Ok(BondType::Double),
            9 => Ok(BondType::String),
            10 => Ok(BondType::Struct),
            11 => Ok(BondType::List),
            12 => Ok(BondType::Set),
            13 => Ok(BondType::Map),
            14 => Ok(BondType::Int8),
            15 => Ok(BondType::Int16),
            16 => Ok(BondType::Int32),
            17 => Ok(BondType::Int64),
            18 => Ok(BondType::WString),
            _ => Err(value),
        }
    }
}
