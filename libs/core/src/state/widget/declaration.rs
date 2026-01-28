use std::collections::HashSet;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use ts_rs::TS;

use crate::resource::ResourceText;

/// The Widget Settings Declaration is a list of configuration definitions.
/// Each definition can be either a group (with nested items) or a direct configuration item.
///
/// This structure is used to render and store widget settings in a user-friendly way,
/// matching the style of the settings window. With this approach, custom configuration
/// windows for specific widgets are not needed.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct WidgetSettingsDeclarationList(Vec<WidgetConfigDefinition>);

impl WidgetSettingsDeclarationList {
    /// Checks if there are duplicate keys in the settings declaration.\
    /// Reserved keys like "enabled" and "$instances" are also checked.
    pub fn there_are_duplicates(&self) -> bool {
        let mut seen: HashSet<&str> = HashSet::new();

        // Reserved keys that cannot be used
        seen.insert("enabled");
        seen.insert("$instances");

        for definition in &self.0 {
            if Self::collect_keys_recursive(definition, &mut seen) {
                return true;
            }
        }
        false
    }

    fn collect_keys_recursive<'a>(
        definition: &'a WidgetConfigDefinition,
        seen: &mut HashSet<&'a str>,
    ) -> bool {
        match definition {
            WidgetConfigDefinition::Group(group) => {
                for item in &group.items {
                    if Self::collect_keys_recursive(item, seen) {
                        return true;
                    }
                }
            }
            WidgetConfigDefinition::Item(item) => {
                let key = item.get_key();
                if seen.contains(key) {
                    return true;
                }
                seen.insert(key);
            }
        }
        false
    }
}

/// A widget configuration definition that can be either a group container or a settings item
#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub enum WidgetConfigDefinition {
    /// A group that contains nested configuration items.
    /// Groups are used to organize related settings with headers.
    Group(WidgetConfigGroup),
    /// A direct configuration item (untagged variant for simpler JSON structure)
    #[serde(untagged)]
    Item(Box<WidgetSettingItem>),
}

/// A group of widget configuration items with a label
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct WidgetConfigGroup {
    /// Label for this group, can use `t::` prefix for translation
    pub label: ResourceText,
    /// Optional description or tooltip for this group
    pub description: Option<ResourceText>,
    /// List of items or nested groups in this group
    pub items: Vec<WidgetConfigDefinition>,
}

impl<'de> Deserialize<'de> for WidgetConfigDefinition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct GroupVariant {
            group: WidgetConfigGroup,
        }

        let value =
            serde_json::Value::deserialize(deserializer).map_err(serde::de::Error::custom)?;

        // Try to deserialize as a group first
        if let Ok(parsed) = GroupVariant::deserialize(value.clone()) {
            return Ok(WidgetConfigDefinition::Group(parsed.group));
        }

        // Otherwise deserialize as an item
        Ok(WidgetConfigDefinition::Item(Box::new(
            serde_json::from_value(value).map_err(serde::de::Error::custom)?,
        )))
    }
}

/// Individual widget setting item with type-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "type")]
pub enum WidgetSettingItem {
    /// Toggle switch for boolean values.\
    /// Renders as a switch/toggle component in the UI.
    #[serde(alias = "switch")]
    Switch(WidgetSettingSwitch),

    /// Selection from a list of options.\
    /// Can be rendered as a dropdown list or inline buttons depending on subtype.
    #[serde(alias = "select")]
    Select(WidgetSettingSelect),

    /// Text input field.\
    /// Supports both single-line and multiline input with optional validation.
    #[serde(alias = "text", alias = "input-text")]
    InputText(WidgetSettingInputText),

    /// Numeric input field.\
    /// Renders as a number input with optional min/max/step constraints.
    #[serde(alias = "number", alias = "input-number")]
    InputNumber(WidgetSettingInputNumber),

    /// Slider/range input for numeric values.\
    /// Renders as a visual slider component.
    #[serde(alias = "range")]
    Range(WidgetSettingRange),

    /// Color picker input.\
    /// Allows users to select colors with optional alpha/transparency support.
    #[serde(alias = "color")]
    Color(WidgetSettingColor),
}

impl WidgetSettingItem {
    /// Returns the unique key identifying this setting item
    pub fn get_key(&self) -> &str {
        match self {
            WidgetSettingItem::Switch(item) => &item.base.key,
            WidgetSettingItem::Select(item) => &item.base.key,
            WidgetSettingItem::InputText(item) => &item.base.key,
            WidgetSettingItem::InputNumber(item) => &item.base.key,
            WidgetSettingItem::Range(item) => &item.base.key,
            WidgetSettingItem::Color(item) => &item.base.key,
        }
    }
}

/// Common fields shared across all widget setting items
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct WidgetSettingBase {
    /// Unique key for this setting, used to identify it in the configuration.\
    /// Must be unique within the widget. Duplicates will be ignored.
    pub key: String,

    /// Label to display to the user
    pub label: ResourceText,

    /// Optional detailed description shown under the label
    pub description: Option<ResourceText>,

    /// Optional tooltip icon with extra information
    pub tip: Option<ResourceText>,

    /// Whether this setting can be configured per monitor in monitor-specific settings
    pub allow_set_by_monitor: bool,

    /// Keys of settings that must be enabled for this item to be active.\
    /// Uses JavaScript truthy logic (!!value) to determine if dependency is met.
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct WidgetSettingSwitch {
    #[serde(flatten)]
    pub base: WidgetSettingBase,
    /// Default value for this switch
    pub default_value: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct WidgetSettingSelect {
    #[serde(flatten)]
    pub base: WidgetSettingBase,
    /// Default selected value (must match one of the option values)
    pub default_value: String,
    /// List of available options
    pub options: Vec<WidgetSelectOption>,
    /// How to render the select options
    pub subtype: WidgetSelectSubtype,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct WidgetSettingInputText {
    #[serde(flatten)]
    pub base: WidgetSettingBase,
    /// Default text value
    pub default_value: String,
    /// Whether to render as a multiline textarea
    pub multiline: bool,
    /// Minimum text length validation
    pub min_length: Option<u32>,
    /// Maximum text length validation
    pub max_length: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct WidgetSettingInputNumber {
    #[serde(flatten)]
    pub base: WidgetSettingBase,
    /// Default numeric value
    pub default_value: f64,
    /// Minimum allowed value
    pub min: Option<f64>,
    /// Maximum allowed value
    pub max: Option<f64>,
    /// Step increment for input controls
    pub step: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct WidgetSettingRange {
    #[serde(flatten)]
    pub base: WidgetSettingBase,
    /// Default value for the range slider
    pub default_value: f64,
    /// Minimum value of the range
    pub min: Option<f64>,
    /// Maximum value of the range
    pub max: Option<f64>,
    /// Step increment for the slider
    pub step: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct WidgetSettingColor {
    #[serde(flatten)]
    pub base: WidgetSettingBase,
    /// Default color value (hex format: #RRGGBB or #RRGGBBAA)
    pub default_value: String,
    /// Whether to allow alpha/transparency channel
    pub allow_alpha: bool,
}

/// An option in a select widget setting
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct WidgetSelectOption {
    /// Optional React icon name to display with this option
    pub icon: Option<String>,
    /// Label to display for this option (can use `t::` prefix)
    pub label: ResourceText,
    /// Value to store when this option is selected (must be unique)
    pub value: String,
}

/// Visual style for rendering select options
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum WidgetSelectSubtype {
    /// Render as a dropdown list (default)
    #[default]
    List,
    /// Render as inline buttons/tabs
    Inline,
}
