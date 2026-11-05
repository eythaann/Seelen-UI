use super::types::*;
use super::value::*;
use super::varint::*;

/// Serializer for Bond CompactBinary v1 payloads.
#[derive(Default)]
pub struct CompactBinaryWriter {
    buf: Vec<u8>,
}

impl CompactBinaryWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    // -- Marshaled header --

    pub fn write_marshaled_header(&mut self) {
        self.buf.extend_from_slice(&COMPACT_BINARY_MAGIC);
        self.buf.extend_from_slice(&COMPACT_BINARY_V1);
    }

    // -- Field headers --

    pub fn write_field_header(&mut self, id: u16, bond_type: BondType) {
        let type_byte = bond_type as u8;
        debug_assert!(type_byte & 0x1F == type_byte);

        if id <= 5 {
            self.buf.push(type_byte | ((id as u8) << 5));
        } else if id <= 0xFF {
            self.buf.push(type_byte | (0x06 << 5));
            self.buf.push(id as u8);
        } else {
            self.buf.push(type_byte | (0x07 << 5));
            self.buf.push(id as u8); // low byte
            self.buf.push((id >> 8) as u8); // high byte
        }
    }

    pub fn write_stop(&mut self) {
        self.buf.push(0x00);
    }

    pub fn write_stop_base(&mut self) {
        self.buf.push(0x01);
    }

    // -- Primitive writers --

    pub fn write_bool(&mut self, val: bool) {
        self.buf.push(val as u8);
    }

    pub fn write_uint8(&mut self, val: u8) {
        self.buf.push(val);
    }

    pub fn write_int8(&mut self, val: i8) {
        self.buf.push(val as u8);
    }

    /// Appends raw bytes directly to the output buffer.
    /// Useful for bulk-writing contiguous fixed-width elements (e.g. list<int8>).
    pub fn write_raw_bytes(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    pub fn write_uint16(&mut self, val: u16) {
        write_varint(&mut self.buf, val as u64);
    }

    pub fn write_int16(&mut self, val: i16) {
        write_varint(&mut self.buf, encode_zigzag_i16(val) as u64);
    }

    pub fn write_uint32(&mut self, val: u32) {
        write_varint(&mut self.buf, val as u64);
    }

    pub fn write_int32(&mut self, val: i32) {
        write_varint(&mut self.buf, encode_zigzag_i32(val) as u64);
    }

    pub fn write_uint64(&mut self, val: u64) {
        write_varint(&mut self.buf, val);
    }

    pub fn write_int64(&mut self, val: i64) {
        write_varint(&mut self.buf, encode_zigzag_i64(val));
    }

    pub fn write_float(&mut self, val: f32) {
        self.buf.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_double(&mut self, val: f64) {
        self.buf.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_string(&mut self, val: &str) {
        self.write_uint32(val.len() as u32);
        self.buf.extend_from_slice(val.as_bytes());
    }

    pub fn write_wstring(&mut self, val: &str) {
        let utf16: Vec<u16> = val.encode_utf16().collect();
        self.write_uint32(utf16.len() as u32);
        for unit in &utf16 {
            self.buf.extend_from_slice(&unit.to_le_bytes());
        }
    }

    // -- Container headers --

    /// Writes a list or set header (v1 format: type byte + varint count).
    pub fn write_container_header(&mut self, element_type: BondType, count: u32) {
        self.buf.push(element_type as u8);
        self.write_uint32(count);
    }

    /// Writes a map header (key type + value type + varint count).
    pub fn write_map_header(&mut self, key_type: BondType, value_type: BondType, count: u32) {
        self.buf.push(key_type as u8);
        self.buf.push(value_type as u8);
        self.write_uint32(count);
    }

    // -- High-level writers --

    /// Writes a single BondValue.
    pub fn write_value(&mut self, val: &BondValue) {
        match val {
            BondValue::Bool(v) => self.write_bool(*v),
            BondValue::UInt8(v) => self.write_uint8(*v),
            BondValue::Int8(v) => self.write_int8(*v),
            BondValue::UInt16(v) => self.write_uint16(*v),
            BondValue::Int16(v) => self.write_int16(*v),
            BondValue::UInt32(v) => self.write_uint32(*v),
            BondValue::Int32(v) => self.write_int32(*v),
            BondValue::UInt64(v) => self.write_uint64(*v),
            BondValue::Int64(v) => self.write_int64(*v),
            BondValue::Float(v) => self.write_float(*v),
            BondValue::Double(v) => self.write_double(*v),
            BondValue::String(v) => self.write_string(v),
            BondValue::WString(v) => self.write_wstring(v),
            BondValue::Struct(s) => self.write_struct(s),
            BondValue::List {
                element_type,
                elements,
            } => {
                self.write_container_header(*element_type, elements.len() as u32);
                for elem in elements {
                    self.write_value(elem);
                }
            }
            BondValue::Set {
                element_type,
                elements,
            } => {
                self.write_container_header(*element_type, elements.len() as u32);
                for elem in elements {
                    self.write_value(elem);
                }
            }
            BondValue::Map {
                key_type,
                value_type,
                entries,
            } => {
                self.write_map_header(*key_type, *value_type, entries.len() as u32);
                for (k, v) in entries {
                    self.write_value(k);
                    self.write_value(v);
                }
            }
        }
    }

    /// Writes a BondStruct (fields + BT_STOP).
    pub fn write_struct(&mut self, s: &BondStruct) {
        for (id, val) in &s.fields {
            self.write_field_header(*id, val.bond_type());
            self.write_value(val);
        }
        self.write_stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::bond::reader::CompactBinaryReader;

    #[test]
    fn write_marshaled_header() {
        let mut w = CompactBinaryWriter::new();
        w.write_marshaled_header();
        assert_eq!(w.into_bytes(), [0x43, 0x42, 0x01, 0x00]);
    }

    #[test]
    fn field_header_small_ids() {
        let mut w = CompactBinaryWriter::new();
        w.write_field_header(0, BondType::Bool);
        w.write_field_header(1, BondType::Struct);
        w.write_field_header(5, BondType::UInt64);
        assert_eq!(w.into_bytes(), [0x02, 0x2A, 0xA6]);
    }

    #[test]
    fn field_header_extended_1byte() {
        let mut w = CompactBinaryWriter::new();
        w.write_field_header(10, BondType::Bool);
        w.write_field_header(40, BondType::Int16);
        assert_eq!(w.into_bytes(), [0xC2, 0x0A, 0xCF, 0x28]);
    }

    #[test]
    fn field_header_extended_2byte() {
        let mut w = CompactBinaryWriter::new();
        w.write_field_header(300, BondType::UInt32);
        assert_eq!(w.into_bytes(), [0xE5, 0x2C, 0x01]);
    }

    #[test]
    fn reader_writer_roundtrip_struct() {
        let original = BondStruct {
            fields: vec![
                (0, BondValue::Bool(true)),
                (1, BondValue::UInt64(1742540908)),
                (10, BondValue::Int16(2790)),
                (
                    20,
                    BondValue::Struct(BondStruct {
                        fields: vec![(0, BondValue::Int8(19)), (1, BondValue::Int8(23))],
                    }),
                ),
            ],
        };

        let mut w = CompactBinaryWriter::new();
        w.write_struct(&original);
        let bytes = w.into_bytes();

        let mut r = CompactBinaryReader::new(&bytes);
        let decoded = r.read_struct().unwrap();
        assert_eq!(r.remaining(), 0);
        assert_eq!(original, decoded);
    }

    #[test]
    fn reader_writer_roundtrip_list() {
        let original = BondStruct {
            fields: vec![(
                0,
                BondValue::List {
                    element_type: BondType::Int8,
                    elements: vec![BondValue::Int8(1), BondValue::Int8(2), BondValue::Int8(3)],
                },
            )],
        };

        let mut w = CompactBinaryWriter::new();
        w.write_struct(&original);
        let bytes = w.into_bytes();

        let mut r = CompactBinaryReader::new(&bytes);
        let decoded = r.read_struct().unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn reader_writer_roundtrip_map() {
        let original = BondStruct {
            fields: vec![(
                0,
                BondValue::Map {
                    key_type: BondType::String,
                    value_type: BondType::Int32,
                    entries: vec![
                        (BondValue::String("hello".into()), BondValue::Int32(42)),
                        (BondValue::String("world".into()), BondValue::Int32(-1)),
                    ],
                },
            )],
        };

        let mut w = CompactBinaryWriter::new();
        w.write_struct(&original);
        let bytes = w.into_bytes();

        let mut r = CompactBinaryReader::new(&bytes);
        let decoded = r.read_struct().unwrap();
        assert_eq!(original, decoded);
    }
}
