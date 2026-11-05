use super::BondError;
use super::types::*;
use super::value::*;
use super::varint::*;

/// Result of reading a field header: either a field with ID+type, or a struct terminator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldHeader {
    Field { id: u16, bond_type: BondType },
    Stop,
    StopBase,
}

/// Deserializer for Bond CompactBinary v1 payloads.
pub struct CompactBinaryReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> CompactBinaryReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    fn ensure(&self, n: usize) -> Result<(), BondError> {
        if self.pos + n > self.data.len() {
            return Err(BondError::UnexpectedEof(self.pos));
        }
        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8, BondError> {
        self.ensure(1)?;
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], BondError> {
        self.ensure(n)?;
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    /// Reads exactly `n` bytes as a borrowed slice, advancing the cursor.
    pub fn read_bytes_slice(&mut self, n: usize) -> Result<&'a [u8], BondError> {
        self.read_bytes(n)
    }

    // -- Marshaled header --

    pub fn read_marshaled_header(&mut self) -> Result<(), BondError> {
        let magic = self.read_bytes(2)?;
        if magic != COMPACT_BINARY_MAGIC {
            return Err(BondError::InvalidHeader);
        }
        let version = self.read_bytes(2)?;
        if version != COMPACT_BINARY_V1 {
            return Err(BondError::InvalidHeader);
        }
        Ok(())
    }

    // -- Field headers --

    pub fn read_field_header(&mut self) -> Result<FieldHeader, BondError> {
        let raw = self.read_byte()?;

        let type_id = raw & 0x1F;
        let id_bits = raw & 0xE0; // upper 3 bits

        if type_id == 0 {
            return if id_bits == 0 {
                Ok(FieldHeader::Stop)
            } else {
                Err(BondError::InvalidTypeId(raw))
            };
        }

        if type_id == 1 && id_bits == 0 {
            return Ok(FieldHeader::StopBase);
        }

        let bond_type =
            BondType::try_from(type_id).map_err(|_| BondError::InvalidTypeId(type_id))?;

        let id = match id_bits {
            0xE0 => {
                let lo = self.read_byte()? as u16;
                let hi = self.read_byte()? as u16;
                (hi << 8) | lo
            }
            0xC0 => self.read_byte()? as u16,
            _ => (id_bits >> 5) as u16,
        };

        Ok(FieldHeader::Field { id, bond_type })
    }

    // -- Primitive readers --

    pub fn read_bool(&mut self) -> Result<bool, BondError> {
        Ok(self.read_byte()? != 0)
    }

    pub fn read_uint8(&mut self) -> Result<u8, BondError> {
        self.read_byte()
    }

    pub fn read_int8(&mut self) -> Result<i8, BondError> {
        Ok(self.read_byte()? as i8)
    }

    pub fn read_uint16(&mut self) -> Result<u16, BondError> {
        let (val, new_pos) = read_varint(self.data, self.pos)?;
        self.pos = new_pos;
        Ok(val as u16)
    }

    pub fn read_int16(&mut self) -> Result<i16, BondError> {
        let (val, new_pos) = read_varint(self.data, self.pos)?;
        self.pos = new_pos;
        Ok(decode_zigzag_i16(val as u16))
    }

    pub fn read_uint32(&mut self) -> Result<u32, BondError> {
        let (val, new_pos) = read_varint(self.data, self.pos)?;
        self.pos = new_pos;
        Ok(val as u32)
    }

    pub fn read_int32(&mut self) -> Result<i32, BondError> {
        let (val, new_pos) = read_varint(self.data, self.pos)?;
        self.pos = new_pos;
        Ok(decode_zigzag_i32(val as u32))
    }

    pub fn read_uint64(&mut self) -> Result<u64, BondError> {
        let (val, new_pos) = read_varint(self.data, self.pos)?;
        self.pos = new_pos;
        Ok(val)
    }

    pub fn read_int64(&mut self) -> Result<i64, BondError> {
        let (val, new_pos) = read_varint(self.data, self.pos)?;
        self.pos = new_pos;
        Ok(decode_zigzag_i64(val))
    }

    pub fn read_float(&mut self) -> Result<f32, BondError> {
        let b = self.read_bytes(4)?;
        Ok(f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn read_double(&mut self) -> Result<f64, BondError> {
        let b = self.read_bytes(8)?;
        Ok(f64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    pub fn read_string(&mut self) -> Result<String, BondError> {
        let len = self.read_uint32()? as usize;
        let bytes = self.read_bytes(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| BondError::InvalidUtf8)
    }

    pub fn read_wstring(&mut self) -> Result<String, BondError> {
        let len = self.read_uint32()? as usize; // number of UTF-16 code units
        let byte_len = len.checked_mul(2).ok_or(BondError::VarintOverflow)?;
        let bytes = self.read_bytes(byte_len)?;
        let utf16: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16(&utf16).map_err(|_| BondError::InvalidUtf16)
    }

    // -- Container headers --

    /// Reads a list or set header. Returns (element_type, count).
    pub fn read_container_header(&mut self) -> Result<(BondType, u32), BondError> {
        let raw = self.read_byte()?;
        let type_id = raw & 0x1F;
        let element_type =
            BondType::try_from(type_id).map_err(|_| BondError::InvalidTypeId(type_id))?;
        let count = self.read_uint32()?;
        Ok((element_type, count))
    }

    /// Reads a map header. Returns (key_type, value_type, count).
    pub fn read_map_header(&mut self) -> Result<(BondType, BondType, u32), BondError> {
        let key_raw = self.read_byte()?;
        let key_type =
            BondType::try_from(key_raw & 0x1F).map_err(|_| BondError::InvalidTypeId(key_raw))?;
        let val_raw = self.read_byte()?;
        let val_type =
            BondType::try_from(val_raw & 0x1F).map_err(|_| BondError::InvalidTypeId(val_raw))?;
        let count = self.read_uint32()?;
        Ok((key_type, val_type, count))
    }

    // -- Skipping --

    /// Advances past a value of the given Bond type without allocating.
    pub fn skip_value(&mut self, bond_type: BondType) -> Result<(), BondError> {
        match bond_type {
            BondType::Bool | BondType::UInt8 | BondType::Int8 => {
                self.read_byte()?;
            }
            BondType::UInt16
            | BondType::UInt32
            | BondType::UInt64
            | BondType::Int16
            | BondType::Int32
            | BondType::Int64 => {
                let (_, new_pos) = read_varint(self.data, self.pos)?;
                self.pos = new_pos;
            }
            BondType::Float => {
                self.read_bytes(4)?;
            }
            BondType::Double => {
                self.read_bytes(8)?;
            }
            BondType::String => {
                let len = self.read_uint32()? as usize;
                self.read_bytes(len)?;
            }
            BondType::WString => {
                let len = self.read_uint32()? as usize;
                let byte_len = len.checked_mul(2).ok_or(BondError::VarintOverflow)?;
                self.read_bytes(byte_len)?;
            }
            BondType::Struct => {
                self.skip_struct()?;
            }
            BondType::List | BondType::Set => {
                let (element_type, count) = self.read_container_header()?;
                for _ in 0..count {
                    self.skip_value(element_type)?;
                }
            }
            BondType::Map => {
                let (key_type, value_type, count) = self.read_map_header()?;
                for _ in 0..count {
                    self.skip_value(key_type)?;
                    self.skip_value(value_type)?;
                }
            }
        }
        Ok(())
    }

    /// Advances past an entire struct (field headers + values) until BT_STOP, without allocating.
    pub fn skip_struct(&mut self) -> Result<(), BondError> {
        loop {
            match self.read_field_header()? {
                FieldHeader::Stop => return Ok(()),
                FieldHeader::StopBase => continue,
                FieldHeader::Field { bond_type, .. } => {
                    self.skip_value(bond_type)?;
                }
            }
        }
    }

    // -- High-level readers --

    /// Reads a single value of the given Bond type.
    pub fn read_value(&mut self, bond_type: BondType) -> Result<BondValue, BondError> {
        match bond_type {
            BondType::Bool => Ok(BondValue::Bool(self.read_bool()?)),
            BondType::UInt8 => Ok(BondValue::UInt8(self.read_uint8()?)),
            BondType::Int8 => Ok(BondValue::Int8(self.read_int8()?)),
            BondType::UInt16 => Ok(BondValue::UInt16(self.read_uint16()?)),
            BondType::Int16 => Ok(BondValue::Int16(self.read_int16()?)),
            BondType::UInt32 => Ok(BondValue::UInt32(self.read_uint32()?)),
            BondType::Int32 => Ok(BondValue::Int32(self.read_int32()?)),
            BondType::UInt64 => Ok(BondValue::UInt64(self.read_uint64()?)),
            BondType::Int64 => Ok(BondValue::Int64(self.read_int64()?)),
            BondType::Float => Ok(BondValue::Float(self.read_float()?)),
            BondType::Double => Ok(BondValue::Double(self.read_double()?)),
            BondType::String => Ok(BondValue::String(self.read_string()?)),
            BondType::WString => Ok(BondValue::WString(self.read_wstring()?)),
            BondType::Struct => Ok(BondValue::Struct(self.read_struct()?)),
            BondType::List => {
                let (element_type, count) = self.read_container_header()?;
                let mut elements = Vec::with_capacity((count as usize).min(self.remaining()));
                for _ in 0..count {
                    elements.push(self.read_value(element_type)?);
                }
                Ok(BondValue::List {
                    element_type,
                    elements,
                })
            }
            BondType::Set => {
                let (element_type, count) = self.read_container_header()?;
                let mut elements = Vec::with_capacity((count as usize).min(self.remaining()));
                for _ in 0..count {
                    elements.push(self.read_value(element_type)?);
                }
                Ok(BondValue::Set {
                    element_type,
                    elements,
                })
            }
            BondType::Map => {
                let (key_type, value_type, count) = self.read_map_header()?;
                let mut entries = Vec::with_capacity((count as usize).min(self.remaining()));
                for _ in 0..count {
                    let k = self.read_value(key_type)?;
                    let v = self.read_value(value_type)?;
                    entries.push((k, v));
                }
                Ok(BondValue::Map {
                    key_type,
                    value_type,
                    entries,
                })
            }
        }
    }

    /// Reads all fields of a struct until BT_STOP, returning a BondStruct.
    pub fn read_struct(&mut self) -> Result<BondStruct, BondError> {
        let mut fields = Vec::new();
        loop {
            match self.read_field_header()? {
                FieldHeader::Stop => break,
                FieldHeader::StopBase => continue,
                FieldHeader::Field { id, bond_type } => {
                    let value = self.read_value(bond_type)?;
                    fields.push((id, value));
                }
            }
        }
        Ok(BondStruct { fields })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_marshaled_header() {
        let data = [0x43, 0x42, 0x01, 0x00];
        let mut reader = CompactBinaryReader::new(&data);
        reader.read_marshaled_header().unwrap();
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn read_marshaled_header_invalid_magic() {
        let data = [0x00, 0x00, 0x01, 0x00];
        let mut reader = CompactBinaryReader::new(&data);
        assert!(reader.read_marshaled_header().is_err());
    }

    #[test]
    fn read_field_header_small_ids() {
        let data = [0x02];
        let mut reader = CompactBinaryReader::new(&data);
        assert_eq!(
            reader.read_field_header().unwrap(),
            FieldHeader::Field {
                id: 0,
                bond_type: BondType::Bool
            }
        );

        let data = [0x2A];
        let mut reader = CompactBinaryReader::new(&data);
        assert_eq!(
            reader.read_field_header().unwrap(),
            FieldHeader::Field {
                id: 1,
                bond_type: BondType::Struct
            }
        );

        let data = [0xA6];
        let mut reader = CompactBinaryReader::new(&data);
        assert_eq!(
            reader.read_field_header().unwrap(),
            FieldHeader::Field {
                id: 5,
                bond_type: BondType::UInt64
            }
        );
    }

    #[test]
    fn read_field_header_extended_1byte() {
        let data = [0xC2, 0x0A];
        let mut reader = CompactBinaryReader::new(&data);
        assert_eq!(
            reader.read_field_header().unwrap(),
            FieldHeader::Field {
                id: 10,
                bond_type: BondType::Bool
            }
        );

        let data = [0xCF, 0x28];
        let mut reader = CompactBinaryReader::new(&data);
        assert_eq!(
            reader.read_field_header().unwrap(),
            FieldHeader::Field {
                id: 40,
                bond_type: BondType::Int16
            }
        );
    }

    #[test]
    fn read_field_header_extended_2byte() {
        let data = [0xE5, 0x2C, 0x01];
        let mut reader = CompactBinaryReader::new(&data);
        assert_eq!(
            reader.read_field_header().unwrap(),
            FieldHeader::Field {
                id: 300,
                bond_type: BondType::UInt32
            }
        );
    }

    #[test]
    fn read_stop() {
        let data = [0x00];
        let mut reader = CompactBinaryReader::new(&data);
        assert_eq!(reader.read_field_header().unwrap(), FieldHeader::Stop);
    }

    #[test]
    fn read_stop_base() {
        let data = [0x01];
        let mut reader = CompactBinaryReader::new(&data);
        assert_eq!(reader.read_field_header().unwrap(), FieldHeader::StopBase);
    }

    #[test]
    fn read_simple_struct() {
        let data = vec![
            0x02, // field 0, BT_BOOL
            0x01, // true
            0x26, // field 1, BT_UINT64
            0x2A, // varint(42)
            0x00, // BT_STOP
        ];

        let mut reader = CompactBinaryReader::new(&data);
        let s = reader.read_struct().unwrap();
        assert_eq!(s.fields.len(), 2);
        assert_eq!(s.fields[0], (0, BondValue::Bool(true)));
        assert_eq!(s.fields[1], (1, BondValue::UInt64(42)));
    }

    #[test]
    fn read_nested_struct() {
        let data = [
            0x0A, // field 0, BT_STRUCT
            0x0E, // field 0, BT_INT8
            0x07, // value = 7
            0x00, // BT_STOP (inner)
            0x00, // BT_STOP (outer)
        ];
        let mut reader = CompactBinaryReader::new(&data);
        let s = reader.read_struct().unwrap();
        assert_eq!(s.fields.len(), 1);
        if let BondValue::Struct(inner) = &s.fields[0].1 {
            assert_eq!(inner.fields.len(), 1);
            assert_eq!(inner.fields[0], (0, BondValue::Int8(7)));
        } else {
            panic!("expected struct");
        }
    }

    #[test]
    fn read_list() {
        let data = [
            0x0B, // field 0, BT_LIST
            0x0E, // element type = BT_INT8
            0x03, // count = 3
            0x01, 0x02, 0x03, // elements
            0x00, // BT_STOP
        ];
        let mut reader = CompactBinaryReader::new(&data);
        let s = reader.read_struct().unwrap();
        assert_eq!(s.fields.len(), 1);
        if let BondValue::List {
            element_type,
            elements,
        } = &s.fields[0].1
        {
            assert_eq!(*element_type, BondType::Int8);
            assert_eq!(elements.len(), 3);
            assert_eq!(elements[0], BondValue::Int8(1));
            assert_eq!(elements[1], BondValue::Int8(2));
            assert_eq!(elements[2], BondValue::Int8(3));
        } else {
            panic!("expected list");
        }
    }

    #[test]
    fn read_truncated_field_header() {
        let data = [];
        let mut reader = CompactBinaryReader::new(&data);
        assert!(reader.read_field_header().is_err());
    }

    #[test]
    fn read_truncated_extended_field_header() {
        let data = [0xC2];
        let mut reader = CompactBinaryReader::new(&data);
        assert!(reader.read_field_header().is_err());
    }

    #[test]
    fn read_truncated_varint() {
        let data = [0x80];
        let mut reader = CompactBinaryReader::new(&data);
        assert!(reader.read_uint64().is_err());
    }

    #[test]
    fn read_string_roundtrip() {
        use crate::utils::bond::writer::CompactBinaryWriter;

        let mut w = CompactBinaryWriter::new();
        w.write_string("hello world");
        let bytes = w.into_bytes();

        let mut r = CompactBinaryReader::new(&bytes);
        assert_eq!(r.read_string().unwrap(), "hello world");
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn read_float_double_roundtrip() {
        use crate::utils::bond::writer::CompactBinaryWriter;

        let mut w = CompactBinaryWriter::new();
        w.write_float(std::f32::consts::PI);
        w.write_double(std::f64::consts::E);
        let bytes = w.into_bytes();

        let mut r = CompactBinaryReader::new(&bytes);
        assert!((r.read_float().unwrap() - std::f32::consts::PI).abs() < f32::EPSILON);
        assert!((r.read_double().unwrap() - std::f64::consts::E).abs() < f64::EPSILON);
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn read_container_count_exceeds_data() {
        let data = [
            0x0B, // field 0, BT_LIST
            0x0E, // element type = BT_INT8
            0xE8, 0x07, // count = 1000 (varint)
            0x01, 0x02, // only 2 elements
            0x00, // BT_STOP
        ];
        let mut reader = CompactBinaryReader::new(&data);
        assert!(reader.read_struct().is_err());
    }
}
