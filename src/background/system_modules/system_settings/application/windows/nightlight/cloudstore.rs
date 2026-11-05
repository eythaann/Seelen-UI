use crate::utils::bond::*;

/// Unwraps a CloudStore binary blob, returning (timestamp, inner_payload_slice).
///
/// Zero-copy: the returned `&[u8]` borrows directly from the input buffer,
/// since Bond `list<int8>` stores elements as contiguous raw bytes.
///
/// The CloudStore wrapper is a marshaled Bond CompactBinary v1 struct:
/// ```text
/// Field 0: struct { field 0: bool = true }   // metadata
/// Field 1: struct {
///   Field 0: uint64 = timestamp              // last-modified Unix seconds
///   Field 1: struct {
///     Field 1: list<int8> = payload           // inner marshaled CB data
///   }
/// }
/// ```
pub fn cloudstore_unwrap(data: &[u8]) -> Result<(u64, &[u8]), BondError> {
    let mut reader = CompactBinaryReader::new(data);
    reader.read_marshaled_header()?;

    let mut timestamp: Option<u64> = None;
    let mut payload: Option<&[u8]> = None;

    // Parse outer struct fields
    loop {
        match reader.read_field_header()? {
            FieldHeader::Stop => break,
            FieldHeader::StopBase => continue,
            FieldHeader::Field {
                id: 0,
                bond_type: BondType::Struct,
            } => {
                // Field 0: metadata struct — skip entirely
                reader.skip_struct()?;
            }
            FieldHeader::Field {
                id: 1,
                bond_type: BondType::Struct,
            } => {
                // Field 1: payload container struct
                loop {
                    match reader.read_field_header()? {
                        FieldHeader::Stop => break,
                        FieldHeader::StopBase => continue,
                        FieldHeader::Field {
                            id: 0,
                            bond_type: BondType::UInt64,
                        } => {
                            timestamp = Some(reader.read_uint64()?);
                        }
                        FieldHeader::Field {
                            id: 1,
                            bond_type: BondType::Struct,
                        } => {
                            // Field 1.1: data wrapper struct
                            loop {
                                match reader.read_field_header()? {
                                    FieldHeader::Stop => break,
                                    FieldHeader::StopBase => continue,
                                    FieldHeader::Field {
                                        id: 1,
                                        bond_type: BondType::List,
                                    } => {
                                        // list<int8> — read header, then borrow raw bytes
                                        let (elem_type, count) = reader.read_container_header()?;
                                        if elem_type != BondType::Int8 {
                                            return Err(BondError::UnexpectedFieldType(1));
                                        }
                                        payload = Some(reader.read_bytes_slice(count as usize)?);
                                    }
                                    FieldHeader::Field { bond_type, .. } => {
                                        reader.skip_value(bond_type)?;
                                    }
                                }
                            }
                        }
                        FieldHeader::Field { bond_type, .. } => {
                            reader.skip_value(bond_type)?;
                        }
                    }
                }
            }
            FieldHeader::Field { bond_type, .. } => {
                reader.skip_value(bond_type)?;
            }
        }
    }

    // Bond's CompactBinary format omits fields holding their default value, so a blob
    // with a zero timestamp legitimately has no field 0 here — default it instead of failing.
    // Likewise an empty inner payload (e.g. Night Light never configured on this account)
    // is a legitimate default and gets omitted entirely, so fall back to an empty marshaled
    // struct (header + immediate stop) rather than erroring.
    const EMPTY_INNER: &[u8] = &[0x43, 0x42, 0x01, 0x00, 0x00];
    let ts = timestamp.unwrap_or(0);
    let bytes = payload.unwrap_or(EMPTY_INNER);
    Ok((ts, bytes))
}

/// Wraps an inner payload into a CloudStore binary blob with the given timestamp.
pub fn cloudstore_wrap(timestamp: u64, inner_payload: &[u8]) -> Vec<u8> {
    let mut writer = CompactBinaryWriter::new();
    writer.write_marshaled_header();

    // Field 0: metadata struct { field 0: bool = true }
    writer.write_field_header(0, BondType::Struct);
    writer.write_field_header(0, BondType::Bool);
    writer.write_bool(true);
    writer.write_stop(); // end metadata struct

    // Field 1: payload container struct
    writer.write_field_header(1, BondType::Struct);

    // Field 1.0: uint64 timestamp
    writer.write_field_header(0, BondType::UInt64);
    writer.write_uint64(timestamp);

    // Field 1.1: data wrapper struct
    writer.write_field_header(1, BondType::Struct);

    // Field 1.1.1: list<int8> = inner payload
    // Int8 elements are stored as contiguous raw bytes — write them in bulk.
    writer.write_field_header(1, BondType::List);
    writer.write_container_header(BondType::Int8, inner_payload.len() as u32);
    writer.write_raw_bytes(inner_payload);

    writer.write_stop(); // end data wrapper struct
    writer.write_stop(); // end payload container struct
    writer.write_stop(); // end outer struct

    writer.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    // The settings test bytes from win-nightlight-lib's nightlight_settings.rs
    const SETTINGS_BYTES: [u8; 60] = [
        0x43, 0x42, 0x01, 0x00, 0x0A, 0x02, 0x01, 0x00, 0x2A, 0x06, 0xEC, 0xA0, 0xF4, 0xBE, 0x06,
        0x2A, 0x2B, 0x0E, 0x26, 0x43, 0x42, 0x01, 0x00, 0x02, 0x01, 0xC2, 0x0A, 0x00, 0xCA, 0x14,
        0x0E, 0x01, 0x2E, 0x0F, 0x00, 0xCA, 0x1E, 0x00, 0xCF, 0x28, 0xCC, 0x2B, 0xCA, 0x32, 0x0E,
        0x13, 0x2E, 0x17, 0x00, 0xCA, 0x3C, 0x0E, 0x07, 0x2E, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    // The state (enabled) test bytes from win-nightlight-lib's nightlight_state.rs
    const STATE_ENABLED_BYTES: [u8; 43] = [
        0x43, 0x42, 0x01, 0x00, 0x0A, 0x02, 0x01, 0x00, 0x2A, 0x06, 0x89, 0x95, 0xFC, 0xBE, 0x06,
        0x2A, 0x2B, 0x0E, 0x15, 0x43, 0x42, 0x01, 0x00, 0x10, 0x00, 0xD0, 0x0A, 0x02, 0xC6, 0x14,
        0xA9, 0xF6, 0xE2, 0xD3, 0xEF, 0xEA, 0xE6, 0xED, 0x01, 0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn unwrap_settings() {
        let (timestamp, inner) = cloudstore_unwrap(&SETTINGS_BYTES).unwrap();
        assert_eq!(timestamp, 1742540908);
        // Inner payload should start with CB header
        assert_eq!(&inner[..4], &[0x43, 0x42, 0x01, 0x00]);
        // Inner payload length = 38 (from list count 0x26)
        assert_eq!(inner.len(), 38);
    }

    #[test]
    fn unwrap_state_enabled() {
        let (timestamp, inner) = cloudstore_unwrap(&STATE_ENABLED_BYTES).unwrap();
        assert_eq!(timestamp, 1742670473);
        assert_eq!(&inner[..4], &[0x43, 0x42, 0x01, 0x00]);
        assert_eq!(inner.len(), 21);
    }

    #[test]
    fn wrap_roundtrip_settings() {
        let (timestamp, inner) = cloudstore_unwrap(&SETTINGS_BYTES).unwrap();
        let rewrapped = cloudstore_wrap(timestamp, inner);
        assert_eq!(rewrapped, SETTINGS_BYTES);
    }

    #[test]
    fn wrap_roundtrip_state_enabled() {
        let (timestamp, inner) = cloudstore_unwrap(&STATE_ENABLED_BYTES).unwrap();
        let rewrapped = cloudstore_wrap(timestamp, inner);
        assert_eq!(rewrapped, STATE_ENABLED_BYTES);
    }

    #[test]
    fn unwrap_missing_payload_defaults_to_empty_inner() {
        // A blob whose data wrapper struct (field 1.1) never got written because its
        // inner list<int8> payload was empty/default — this is what a Night Light
        // registry blob looks like on an account that has never touched the feature.
        let mut writer = CompactBinaryWriter::new();
        writer.write_marshaled_header();

        writer.write_field_header(0, BondType::Struct);
        writer.write_stop();

        writer.write_field_header(1, BondType::Struct);
        writer.write_field_header(0, BondType::UInt64);
        writer.write_uint64(1234);
        writer.write_stop();

        writer.write_stop();
        let data = writer.into_bytes();

        let (timestamp, inner) = cloudstore_unwrap(&data).unwrap();
        assert_eq!(timestamp, 1234);

        // The empty inner payload should still parse as a valid, empty CB struct.
        let mut reader = CompactBinaryReader::new(inner);
        reader.read_marshaled_header().unwrap();
        assert_eq!(reader.read_field_header().unwrap(), FieldHeader::Stop);
    }
}
