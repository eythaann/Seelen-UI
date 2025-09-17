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

        let mut category_enum = vec![
        "#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS)]".to_string(),
        "#[repr(u16)]".to_string(),
        "pub enum BLEAppearanceCategory {".to_string(),
        "    #[default]".to_string(),
    ];

        let mut appearance_enum = vec![
            "#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, TS)]".to_string(),
            "#[serde(tag = \"category\", content = \"subcategory\")]".to_string(),
            "pub enum BLEAppearance {".to_string(),
        ];

        let mut appearance_impl = vec![
            "impl From<u16> for BLEAppearance {".to_string(),
            "    fn from(value: u16) -> Self {".to_string(),
            "        let category = BLEAppearanceCategory::from(value >> 6); // 10 bits"
                .to_string(),
            "        let subcategory = value & 0b111111; // 6 bits\n".to_string(),
            "        match category {".to_string(),
        ];

        for category in definition.appearance_values {
            let name = regex.replace_all(&category.name, "");
            let sub_enum_name = format!("BLEAppearance{name}SubCategory");

            category_enum.push(format!("    {name} = 0x{:x},", category.category));
            appearance_enum.push(format!("    {name}({sub_enum_name}),"));
            appearance_impl.push(format!("            BLEAppearanceCategory::{name} => BLEAppearance::{name}({sub_enum_name}::from(subcategory)),"));

            let mut subcategory_enum = vec![
            "#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, IntoPrimitive, Serialize, Deserialize, TS)]".to_string(),
            "#[cfg_attr(feature = \"gen-binds\", ts(export_to = \"BLEAppearanceSubCategory.ts\"))]".to_string(),
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

        category_enum.push("}\n\n".to_string());
        file.write_all(category_enum.join("\n").as_bytes())?;

        appearance_enum.push("}\n\n".to_string());
        file.write_all(appearance_enum.join("\n").as_bytes())?;

        appearance_impl.push("        }".to_string());
        appearance_impl.push("    }".to_string());
        appearance_impl.push("}".to_string());
        file.write_all(appearance_impl.join("\n").as_bytes())?;

        Ok(())
    }
}
