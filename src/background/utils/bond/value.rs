use super::types::BondType;

/// A self-describing Bond value. Used for generic parsing when the schema is not known
/// at compile time, or for preserving fields during roundtrip serialization.
#[derive(Debug, Clone, PartialEq)]
pub enum BondValue {
    Bool(bool),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float(f32),
    Double(f64),
    String(String),
    WString(String),
    Struct(BondStruct),
    List {
        element_type: BondType,
        elements: Vec<BondValue>,
    },
    Set {
        element_type: BondType,
        elements: Vec<BondValue>,
    },
    Map {
        key_type: BondType,
        value_type: BondType,
        entries: Vec<(BondValue, BondValue)>,
    },
}

impl BondValue {
    /// Returns the BondType corresponding to this value.
    pub fn bond_type(&self) -> BondType {
        match self {
            BondValue::Bool(_) => BondType::Bool,
            BondValue::UInt8(_) => BondType::UInt8,
            BondValue::UInt16(_) => BondType::UInt16,
            BondValue::UInt32(_) => BondType::UInt32,
            BondValue::UInt64(_) => BondType::UInt64,
            BondValue::Int8(_) => BondType::Int8,
            BondValue::Int16(_) => BondType::Int16,
            BondValue::Int32(_) => BondType::Int32,
            BondValue::Int64(_) => BondType::Int64,
            BondValue::Float(_) => BondType::Float,
            BondValue::Double(_) => BondType::Double,
            BondValue::String(_) => BondType::String,
            BondValue::WString(_) => BondType::WString,
            BondValue::Struct(_) => BondType::Struct,
            BondValue::List { .. } => BondType::List,
            BondValue::Set { .. } => BondType::Set,
            BondValue::Map { .. } => BondType::Map,
        }
    }
}

/// An ordered collection of Bond struct fields. Fields are stored as (field_id, value) pairs
/// and should be sorted by field ID for correct serialization.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BondStruct {
    pub fields: Vec<(u16, BondValue)>,
}

impl BondStruct {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    /// Looks up the first field with the given ID.
    pub fn get(&self, id: u16) -> Option<&BondValue> {
        self.fields
            .iter()
            .find(|(fid, _)| *fid == id)
            .map(|(_, v)| v)
    }

    /// Returns true if a field with the given ID exists.
    pub fn has(&self, id: u16) -> bool {
        self.fields.iter().any(|(fid, _)| *fid == id)
    }

    /// Adds a field. Fields should be added in ascending ID order.
    pub fn push(&mut self, id: u16, value: BondValue) {
        self.fields.push((id, value));
    }
}
