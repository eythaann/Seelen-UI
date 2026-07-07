#[cfg(feature = "gen-binds")]
mod tests {
    #[derive(Deserialize)]
    struct AppearanceDefinition {
        pub appearance_values: Vec<AppearanceCategoryDefinition>,
    }

    #[derive(Deserialize)]
    struct AppearanceCategoryDefinition {
        pub category: u16,
        pub name: String,
        #[serde(default)]
        pub subcategory: Vec<AppearanceSubcategoryDefinition>,
    }

    #[derive(Deserialize)]
    struct AppearanceSubcategoryDefinition {
        pub value: u16,
        pub name: String,
    }

    #[test]
    fn build_low_energy_enums() -> crate::error::Result<()> {
        use regex::Regex;
        use std::io::Write;

        let yaml_str = include_str!("appearance_values.yml");
        let definition: AppearanceDefinition = serde_yaml::from_str(yaml_str)?;

        let regex = Regex::new(r"[^a-zA-Z0-9_]").unwrap();

        let mut file = std::fs::File::create("./src/system_state/bluetooth/low_energy_enums.rs")?;
        file.write_all(
            "// This file was generated via rust macros. Don't modify manually.\n".as_bytes(),
        )?;
        file.write_all(
        "// all this structs are based on official docs https://www.bluetooth.com/specifications/assigned-numbers\n\n".as_bytes()
        )?;

        let mut appearance_enum = vec![
            "#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]".to_string(),
            "#[cfg_attr(all(feature = \"gen-binds\", not(feature = \"salvo\")), derive(ts_rs::TS))]"
                .to_string(),
            "#[serde(tag = \"category\", content = \"subcategory\")]".to_string(),
            "pub enum BLEAppearance {".to_string(),
        ];

        let mut appearance_impl = vec![
            "impl From<u16> for BLEAppearance {".to_string(),
            "    fn from(value: u16) -> Self {".to_string(),
            "        let category = value >> 6; // 10 bits".to_string(),
            "        let subcategory = value & 0b111111; // 6 bits\n".to_string(),
            "        match category {".to_string(),
        ];

        for category in definition.appearance_values {
            let name = regex.replace_all(&category.name, "");
            let sub_enum_name = format!("BLEAppearance{name}SubCategory");

            appearance_enum.push(format!("    {name}({sub_enum_name}),"));
            appearance_impl.push(format!(
                "            0x{:x} => BLEAppearance::{name}({sub_enum_name}::from(subcategory)),",
                category.category
            ));

            let mut subcategory_enum = vec![
            "#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize)]".to_string(),
            "#[cfg_attr(all(feature = \"gen-binds\", not(feature = \"salvo\")), derive(ts_rs::TS))]".to_string(),
            "#[cfg_attr(all(feature = \"gen-binds\", not(feature = \"salvo\")), ts(export_to = \"BLEAppearanceSubCategory.ts\"))]".to_string(),
            "#[repr(u16)]".to_string(),
            format!("pub enum {sub_enum_name} {{"),
        ];

            for subcategory in category.subcategory {
                let sub_name = regex.replace_all(&subcategory.name, "");
                subcategory_enum.push(format!("    {} = 0x{:x},", sub_name, subcategory.value));
            }

            subcategory_enum.push("    #[num_enum(catch_all)]".to_string());
            subcategory_enum.push("    Reserved(u16),".to_string());
            subcategory_enum.push("}\n\n".to_string());

            file.write_all(subcategory_enum.join("\n").as_bytes())?;
        }

        appearance_enum.push("}\n\n".to_string());
        file.write_all(appearance_enum.join("\n").as_bytes())?;

        appearance_impl.push(
            "            _ => BLEAppearance::Unknown(BLEAppearanceUnknownSubCategory::from(subcategory)),"
                .to_string(),
        );
        appearance_impl.push("        }".to_string());
        appearance_impl.push("    }".to_string());
        appearance_impl.push("}".to_string());
        file.write_all(appearance_impl.join("\n").as_bytes())?;

        Ok(())
    }

    #[derive(Deserialize)]
    struct ClassOfDeviceDefinition {
        pub cod_services: Vec<CodServiceDefinition>,
        pub cod_device_class: Vec<CodDeviceClassDefinition>,
    }

    #[derive(Deserialize)]
    struct CodServiceDefinition {
        pub bit: u8,
        pub name: String,
    }

    #[derive(Deserialize)]
    struct CodDeviceClassDefinition {
        pub major: u8,
        pub name: String,
        #[serde(default)]
        pub subsplit: Option<u8>,
        #[serde(default)]
        pub minor: Vec<CodMinorDefinition>,
        #[serde(default)]
        pub minor_bits: Vec<CodMinorDefinition>,
        #[serde(default)]
        pub subminor: Vec<CodMinorDefinition>,
    }

    #[derive(Deserialize)]
    struct CodMinorDefinition {
        pub value: u8,
        pub name: String,
    }

    /// Keeps only the part of the name before any parenthetical example list,
    /// then strips every character that isn't a valid identifier character.
    fn sanitize_name(name: &str, regex: &regex::Regex) -> String {
        let short = name.split('(').next().unwrap_or(name).trim();
        let sanitized = regex.replace_all(short, "").to_string();
        if sanitized.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            format!("N{sanitized}")
        } else {
            sanitized
        }
    }

    /// Appends the hex value to a variant name that already appeared earlier
    /// in the same enum, since some official names repeat (e.g. multiple
    /// "Reserved for Future Use" entries with different values).
    fn dedup_variant_name(
        name: String,
        used: &mut std::collections::HashSet<String>,
        value: u8,
    ) -> String {
        if used.insert(name.clone()) {
            return name;
        }
        format!("{name}0x{value:x}")
    }

    fn write_value_enum(
        file: &mut std::fs::File,
        name: &str,
        values: &[CodMinorDefinition],
        regex: &regex::Regex,
    ) -> crate::error::Result<()> {
        use std::io::Write;

        let mut lines = vec![
        "#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize)]".to_string(),
        "#[cfg_attr(all(feature = \"gen-binds\", not(feature = \"salvo\")), derive(ts_rs::TS))]".to_string(),
        "#[repr(u8)]".to_string(),
        format!("pub enum {name} {{"),
    ];

        let mut used = std::collections::HashSet::new();
        for value in values {
            let variant_name =
                dedup_variant_name(sanitize_name(&value.name, regex), &mut used, value.value);
            lines.push(format!("    {variant_name} = 0x{:x},", value.value));
        }

        lines.push("    #[num_enum(catch_all)]".to_string());
        lines.push("    Reserved(u8),".to_string());
        lines.push("}\n\n".to_string());
        file.write_all(lines.join("\n").as_bytes())?;
        Ok(())
    }

    #[test]
    fn build_class_of_device_enums() -> crate::error::Result<()> {
        use regex::Regex;
        use std::io::Write;

        let yaml_str = include_str!("class_of_device.yml");
        let definition: ClassOfDeviceDefinition = serde_yaml::from_str(yaml_str)?;

        let regex = Regex::new(r"[^a-zA-Z0-9_]").unwrap();

        let mut file =
            std::fs::File::create("./src/system_state/bluetooth/class_of_device_enums.rs")?;
        file.write_all(
            "// This file was generated via rust macros. Don't modify manually.\n".as_bytes(),
        )?;
        file.write_all(
            "// all this structs are based on official docs https://bitbucket.org/bluetooth-SIG/public/src/main/assigned_numbers/core/class_of_device.yaml\n\n".as_bytes()
        )?;

        // -- Major Service Class (bit flags relative to the lowest declared bit) --
        let min_service_bit = definition
            .cod_services
            .iter()
            .map(|s| s.bit)
            .min()
            .unwrap_or(0);

        let mut service_names = Vec::new();
        let mut service_enum = vec![
            "#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]".to_string(),
            "#[cfg_attr(all(feature = \"gen-binds\", not(feature = \"salvo\")), derive(ts_rs::TS))]"
                .to_string(),
            "#[repr(u32)]".to_string(),
            "pub enum BluetoothMajorServiceClass {".to_string(),
        ];
        for service in &definition.cod_services {
            let name = sanitize_name(&service.name, &regex);
            let value = 1u32 << (service.bit - min_service_bit);
            service_enum.push(format!("    {name} = 0x{value:x},"));
            service_names.push(name);
        }
        service_enum.push("}\n\n".to_string());
        file.write_all(service_enum.join("\n").as_bytes())?;

        let service_from_bits = vec![
            "impl BluetoothMajorServiceClass {".to_string(),
            "    /// `bits` should already be shifted so the lowest declared service bit is bit 0"
                .to_string(),
            "    pub fn from_bits(bits: u32) -> Vec<Self> {".to_string(),
            "        use BluetoothMajorServiceClass::*;".to_string(),
            format!("        [{}]", service_names.join(", ")),
            "            .into_iter()".to_string(),
            "            .filter(|&service| bits & service as u32 != 0)".to_string(),
            "            .collect()".to_string(),
            "    }".to_string(),
            "}\n\n".to_string(),
        ];
        file.write_all(service_from_bits.join("\n").as_bytes())?;

        // -- Class (major + minor + subminor merged into a single tagged enum) --
        let mut class_enum = vec![
            "#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]".to_string(),
            "#[cfg_attr(all(feature = \"gen-binds\", not(feature = \"salvo\")), derive(ts_rs::TS))]"
                .to_string(),
            "#[serde(tag = \"major\")]".to_string(),
            "pub enum BluetoothClass {".to_string(),
        ];
        let mut class_from = vec![
            "impl BluetoothClass {".to_string(),
            "    /// `major` must already be masked down to its 5 bits and `minor` to its 6 bits"
                .to_string(),
            "    pub fn from_major_and_minor(major: u8, minor: u8) -> Self {".to_string(),
            "        match major {".to_string(),
        ];

        for class in &definition.cod_device_class {
            let major_name = sanitize_name(&class.name, &regex);
            let major_value = class.major;

            if !class.minor_bits.is_empty() {
                let subsplit = class.subsplit.unwrap_or(6 - class.minor_bits.len() as u8);
                let min_bit = class.minor_bits.iter().map(|m| m.value).min().unwrap_or(0);
                let flags_name = format!("Bluetooth{major_name}Minor");

                let mut flags_enum = vec![
                    "#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, Serialize, Deserialize)]".to_string(),
                    "#[cfg_attr(all(feature = \"gen-binds\", not(feature = \"salvo\")), derive(ts_rs::TS))]".to_string(),
                    "#[repr(u8)]".to_string(),
                    format!("pub enum {flags_name} {{"),
                ];
                for m in &class.minor_bits {
                    let name = sanitize_name(&m.name, &regex);
                    flags_enum.push(format!("    {name} = 0x{:x},", 1u8 << (m.value - min_bit)));
                }
                flags_enum.push("}\n\n".to_string());
                file.write_all(flags_enum.join("\n").as_bytes())?;

                let lower_bits = 6 - subsplit;
                let upper_mask = (1u8 << subsplit) - 1;
                let lower_mask = (1u8 << lower_bits) - 1;
                let flag_names = class
                    .minor_bits
                    .iter()
                    .map(|m| sanitize_name(&m.name, &regex))
                    .collect::<Vec<_>>()
                    .join(", ");

                if !class.subminor.is_empty() {
                    let subminor_name = format!("Bluetooth{major_name}SubMinor");
                    write_value_enum(&mut file, &subminor_name, &class.subminor, &regex)?;
                    class_enum.push(format!(
                        "    {major_name} {{ minor: Vec<{flags_name}>, subminor: {subminor_name} }},"
                    ));
                    class_from.push(format!(
                        "            0x{major_value:x} => {{
                let upper = (minor >> {lower_bits}) & 0x{upper_mask:x};
                let lower = minor & 0x{lower_mask:x};
                use {flags_name}::*;
                let flags = [{flag_names}]
                    .into_iter()
                    .filter(|&flag| upper & flag as u8 != 0)
                    .collect();
                BluetoothClass::{major_name} {{ minor: flags, subminor: {subminor_name}::from(lower) }}
            }}"
                    ));
                } else {
                    class_enum.push(format!("    {major_name} {{ minor: Vec<{flags_name}> }},"));
                    class_from.push(format!(
                        "            0x{major_value:x} => {{
                let upper = (minor >> {lower_bits}) & 0x{upper_mask:x};
                use {flags_name}::*;
                let flags = [{flag_names}]
                    .into_iter()
                    .filter(|&flag| upper & flag as u8 != 0)
                    .collect();
                BluetoothClass::{major_name} {{ minor: flags }}
            }}"
                    ));
                }
            } else if let Some(subsplit) = class.subsplit {
                let minor_name = format!("Bluetooth{major_name}Minor");
                let subminor_name = format!("Bluetooth{major_name}SubMinor");
                write_value_enum(&mut file, &minor_name, &class.minor, &regex)?;
                write_value_enum(&mut file, &subminor_name, &class.subminor, &regex)?;
                class_enum.push(format!(
                    "    {major_name} {{ minor: {minor_name}, subminor: {subminor_name} }},"
                ));

                let lower_bits = 6 - subsplit;
                let upper_mask = (1u8 << subsplit) - 1;
                let lower_mask = (1u8 << lower_bits) - 1;
                class_from.push(format!(
                    "            0x{major_value:x} => {{
                let upper = (minor >> {lower_bits}) & 0x{upper_mask:x};
                let lower = minor & 0x{lower_mask:x};
                BluetoothClass::{major_name} {{ minor: {minor_name}::from(upper), subminor: {subminor_name}::from(lower) }}
            }}"
                ));
            } else if !class.minor.is_empty() {
                let minor_name = format!("Bluetooth{major_name}Minor");
                write_value_enum(&mut file, &minor_name, &class.minor, &regex)?;
                class_enum.push(format!("    {major_name} {{ minor: {minor_name} }},"));
                class_from.push(format!(
                    "            0x{major_value:x} => BluetoothClass::{major_name} {{ minor: {minor_name}::from(minor) }},"
                ));
            } else {
                class_enum.push(format!("    {major_name} {{ minor: u8 }},"));
                class_from.push(format!(
                    "            0x{major_value:x} => BluetoothClass::{major_name} {{ minor }},"
                ));
            }
        }

        class_enum.push("    Reserved { minor: u8 },".to_string());
        class_enum.push("}\n\n".to_string());
        file.write_all(class_enum.join("\n").as_bytes())?;

        class_from.push("            _ => BluetoothClass::Reserved { minor },".to_string());
        class_from.push("        }".to_string());
        class_from.push("    }".to_string());
        class_from.push("}".to_string());
        file.write_all(class_from.join("\n").as_bytes())?;

        Ok(())
    }
}
