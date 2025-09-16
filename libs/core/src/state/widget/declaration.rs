use std::collections::HashSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::resource::ResourceText;

/// Wsd = Widget Settings Declaration
macro_rules! wsd_item {
    (
        @struct
        #[subtypes($($subtype:ident),*)]
        struct $name:ident {
            $($rest:tt)*
        }
    ) => {
        paste::paste! {
            wsd_item! {
                @struct
                struct $name {
                    subtype: [<WsdItem $name Subtype>],
                    $($rest)*
                }
            }

            #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
            pub enum [<WsdItem $name Subtype>] {
                $(
                    $subtype,
                )*
            }
        }
    };
    (
        @struct
        struct $name:ident {
            $($rest:tt)*
        }
    ) => {
        paste::paste! {
            #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
            #[serde(rename_all = "camelCase")]
            pub struct [<WsdItem $name>] {
                /// Unique key for this item, used to identify it in the settings, should be unique.\
                /// If duplicated, duplicated items will be ignored.
                key: String,
                /// Label for this item could start with the prefix `t::` for translation
                label: ResourceText,
                /// This setting could be set by monitor on the settings by monitor section.
                #[serde(default)]
                allow_set_by_monitor: bool,
                /// Keys of items to be set in order to enable this item.
                ///
                /// it uses js logic (!!value) to determine if the item is enabled
                #[serde(default)]
                dependencies: Vec<String>,
                $($rest)*
            }
        }
    };
    (
        $(
            $(#[subtypes($($subtype:ident),*)])?
            struct $name:ident {
                $($rest:tt)*
            }
        )*
    ) => {
        $(
            wsd_item! {
                @struct
                $(#[subtypes($($subtype),*)])?
                struct $name {
                    $($rest)*
                }
            }
        )*

        paste::paste! {
            #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
            #[serde(tag = "type", rename_all = "kebab-case")]
            pub enum WsdItem {
                $(
                    $name([<WsdItem $name>]),
                )*
            }

            impl WsdItem {
                pub fn get_key(&self) -> &str {
                    match self {
                        $(
                            WsdItem::$name(item) => &item.key,
                        )*
                    }
                }
            }
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
pub struct WsdItemSelectOption {
    /// react icon name
    icon: Option<String>,
    /// The label to be displayed
    label: ResourceText,
    /// The value to be set when this option is selected, should be unique
    value: String,
}

wsd_item! {
    struct Switch {
        #[serde(default)]
        default_value: bool,
    }

    #[subtypes(List, Inline)]
    struct Select {
        #[serde(default)]
        default_value: String,
        options: Vec<WsdItemSelectOption>,
    }

    struct InputText {
        #[serde(default)]
        default_value: String,
    }

    struct InputNumber {
        #[serde(default)]
        default_value: f64,
        min: Option<f64>,
        max: Option<f64>,
    }

    struct Range {
        #[serde(default)]
        default_value: f64,
        from: f64,
        to: f64,
        step: Option<f64>,
    }

    struct Color {
        #[serde(default)]
        default_value: String,
        allow_alpha: bool,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct WsdGroupEntry {
    /// Declaration of key and value to be used as configuration.
    /// If children is not empty, this will beused as header of the subgroup.
    config: WsdItem,
    /// List of items in this subgroup
    #[serde(default)]
    children: Vec<WsdGroupEntry>,
}

impl WsdGroupEntry {
    fn get_keys(&self) -> Vec<&str> {
        let mut keys = Vec::new();
        keys.push(self.config.get_key());
        for entry in &self.children {
            keys.append(&mut entry.get_keys());
        }
        keys
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct WsdGroup {
    /// List of items in this group
    group: Vec<WsdGroupEntry>,
}

impl WsdGroup {
    fn get_keys(&self) -> Vec<&str> {
        let mut keys = Vec::new();
        for entry in &self.group {
            keys.append(&mut entry.get_keys());
        }
        keys
    }
}

/// The Widget Settings Declaration is a list of groups
/// each group is a list of items and can have subgroups with headers.
///
/// This declarations will be used to render and store the settings of the widget on a
/// friendy way for the users, also will match the styles of the settings window.
///
/// With this, custom windows or widgets to configure an specific widget are not needed.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct WidgetSettingsDeclarationList(Vec<WsdGroup>);

impl WidgetSettingsDeclarationList {
    pub fn there_are_duplicates(&self) -> bool {
        let mut seen: HashSet<&str> = HashSet::new();

        // reserved keys
        seen.insert("enabled");
        seen.insert("$instances");

        for group in &self.0 {
            for key in group.get_keys() {
                if seen.contains(key) {
                    return true;
                }
                seen.insert(key);
            }
        }
        false
    }
}
