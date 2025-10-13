use std::{collections::HashSet, path::PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct WegAppGroupItem {
    pub handle: isize,
    pub title: String,
    pub is_iconic: bool,
    pub is_zoomed: bool,
    /// last time the app was active, timestamp in milliseconds,
    /// could be 0 if we don't know when the app was actived
    pub last_active: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum WegItemSubtype {
    File,
    Folder,
    App,
    /// this is used for backward compatibility, will be removed in v3
    #[default]
    UnknownV2_1_6,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(untagged)]
pub enum RelaunchArguments {
    Array(Vec<String>),
    String(String),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct PinnedWegItemData {
    /// internal UUID to differentiate items
    pub id: String,
    /// Subtype of the item (mandatory, but is optional for backward compatibility)
    pub subtype: WegItemSubtype,
    /// Application user model id.
    pub umid: Option<String>,
    /// path to file, forder or program.
    pub path: PathBuf,
    /// @deprecaed will be removed in v3, use relaunch_program instead.
    #[ts(skip)]
    #[deprecated]
    #[serde(skip_serializing)]
    pub relaunch_command: Option<String>,
    /// program to be executed
    pub relaunch_program: String,
    /// arguments to be passed to the relaunch program
    pub relaunch_args: Option<RelaunchArguments>,
    /// path where ejecute the relaunch command
    pub relaunch_in: Option<PathBuf>,
    /// display name of the item
    pub display_name: String,
    ///@deprecaed will be removed in v3, use subtype `Folder` instead.
    #[ts(skip)]
    #[serde(skip_serializing)]
    #[deprecated]
    pub is_dir: bool,
    /// Window handles in the app group, in case of pinned file/dir always will be empty
    #[serde(skip_deserializing)]
    pub windows: Vec<WegAppGroupItem>,
    /// This intention is to prevent pinned state change, when this is neccesary
    #[serde(skip_deserializing)]
    pub pin_disabled: bool,
}

impl PinnedWegItemData {
    pub fn set_pin_disabled(&mut self, pin_disabled: bool) {
        self.pin_disabled = pin_disabled;
    }

    /// Some apps changes of place on update, commonly this contains an App User Model Id
    /// the path should be updated to the new location on these cases.
    pub fn should_ensure_path(&self) -> bool {
        self.umid.is_none() || self.path.extension().is_some_and(|ext| ext == "lnk")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "type")]
pub enum WegItem {
    #[serde(alias = "PinnedApp")]
    Pinned(PinnedWegItemData),
    Temporal(PinnedWegItemData),
    Separator {
        id: String,
    },
    Media {
        id: String,
    },
    StartMenu {
        id: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export, repr(enum = name))]
pub enum WegItemType {
    Pinned,
    Temporal,
    Separator,
    Media,
    StartMenu,
}

impl WegItem {
    pub fn id(&self) -> &String {
        match self {
            WegItem::Pinned(data) => &data.id,
            WegItem::Temporal(data) => &data.id,
            WegItem::Separator { id } => id,
            WegItem::Media { id } => id,
            WegItem::StartMenu { id } => id,
        }
    }

    fn set_id(&mut self, identifier: String) {
        match self {
            WegItem::Pinned(data) => data.id = identifier,
            WegItem::Temporal(data) => data.id = identifier,
            WegItem::Separator { id } => *id = identifier,
            WegItem::Media { id } => *id = identifier,
            WegItem::StartMenu { id } => *id = identifier,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct WegItems {
    /// Whether the reordering possible on the weg
    pub is_reorder_disabled: bool,
    pub left: Vec<WegItem>,
    pub center: Vec<WegItem>,
    pub right: Vec<WegItem>,
}

#[allow(deprecated)]
impl Default for WegItems {
    fn default() -> Self {
        Self {
            is_reorder_disabled: false,
            left: vec![WegItem::StartMenu { id: String::new() }],
            center: vec![WegItem::Pinned(PinnedWegItemData {
                id: String::new(),
                umid: None,
                subtype: WegItemSubtype::App,
                path: "C:\\Windows\\explorer.exe".into(),
                display_name: "Explorer".into(),
                relaunch_command: None,
                relaunch_program: "C:\\Windows\\explorer.exe".into(),
                relaunch_args: None,
                relaunch_in: None,
                is_dir: false,
                windows: vec![],
                pin_disabled: false,
            })],
            right: vec![WegItem::Media { id: String::new() }],
        }
    }
}

#[allow(deprecated)]
impl WegItems {
    fn get_parts_of_deprecated_inline_command(cmd: &str) -> (String, String) {
        let start_double_quoted = cmd.starts_with("\"");
        if start_double_quoted || cmd.starts_with("'") {
            let delimiter = if start_double_quoted { '"' } else { '\'' };
            let mut parts = cmd.split(['"', '\'']).filter(|s| !s.is_empty());

            let program = parts.next().unwrap_or_default().trim().to_owned();
            let args = cmd
                .trim_start_matches(&format!("{delimiter}{program}{delimiter}"))
                .trim()
                .to_owned();
            (program, args)
        } else {
            (cmd.trim().to_string(), String::new())
        }
    }

    fn sanitize_items(dict: &mut HashSet<String>, items: Vec<WegItem>) -> Vec<WegItem> {
        let mut result = Vec::new();
        for mut item in items {
            match &mut item {
                WegItem::Pinned(data) => {
                    if data.path.as_os_str().is_empty()
                        || (data.should_ensure_path() && !data.path.exists())
                    {
                        continue;
                    }

                    // migration step for items before v2.1.6
                    if data.subtype == WegItemSubtype::UnknownV2_1_6 {
                        data.subtype = if data.is_dir {
                            WegItemSubtype::Folder
                        } else if data
                            .relaunch_command
                            .as_ref()
                            .is_some_and(|r| r.to_lowercase().contains(".exe"))
                        {
                            WegItemSubtype::App
                        } else {
                            WegItemSubtype::File
                        };
                    }

                    // migration step for items before v2.2.6
                    if let Some(old_command) = data.relaunch_command.take() {
                        if data.relaunch_program.is_empty() {
                            let (program, args) =
                                Self::get_parts_of_deprecated_inline_command(&old_command);
                            data.relaunch_program = program;
                            if !args.is_empty() {
                                data.relaunch_args = Some(RelaunchArguments::String(args));
                            }
                        }
                    }

                    if data.relaunch_program.is_empty() {
                        data.relaunch_program = data.path.to_string_lossy().to_string();
                    }
                }
                WegItem::Temporal(data) => {
                    if data.path.as_os_str().is_empty()
                        || data.windows.is_empty()
                        || (data.should_ensure_path() && !data.path.exists())
                    {
                        continue;
                    }
                    if data.relaunch_program.is_empty() {
                        data.relaunch_program = data.path.to_string_lossy().to_string();
                    }
                }
                _ => {}
            }

            if item.id().is_empty() {
                item.set_id(uuid::Uuid::new_v4().to_string());
            }

            if !dict.contains(item.id()) {
                dict.insert(item.id().clone());
                result.push(item);
            }
        }
        result
    }

    pub fn sanitize(&mut self) {
        let mut dict = HashSet::new();
        self.left = Self::sanitize_items(&mut dict, std::mem::take(&mut self.left));
        self.center = Self::sanitize_items(&mut dict, std::mem::take(&mut self.center));
        self.right = Self::sanitize_items(&mut dict, std::mem::take(&mut self.right));
    }
}

#[cfg(test)]
mod tests {
    use crate::state::WegItems;

    #[test]
    fn should_return_empty_response_for_empty_command() {
        let (program, args) = WegItems::get_parts_of_deprecated_inline_command("");
        assert_eq!(program, "");
        assert_eq!(args, "");
    }

    #[test]
    fn should_parse_a_simple_command_without_arguments() {
        let (program, args) = WegItems::get_parts_of_deprecated_inline_command("node");
        assert_eq!(program, "node");
        assert_eq!(args, "");
    }

    #[test]
    fn should_parse_a_quoted_program_path_without_splitting_args() {
        let (program, args) = WegItems::get_parts_of_deprecated_inline_command(
            "\"C:\\Program Files\\node.exe\" script.js",
        );
        assert_eq!(program, "C:\\Program Files\\node.exe");
        assert_eq!(args, "script.js");
    }

    #[test]
    fn should_parse_a_single_quoted_program_path_without_splitting_args() {
        let (program, args) =
            WegItems::get_parts_of_deprecated_inline_command("'/usr/local/bin/node' script.js");
        assert_eq!(program, "/usr/local/bin/node");
        assert_eq!(args, "script.js");
    }

    #[test]
    fn should_handle_program_path_with_spaces_without_quotes() {
        let (program, args) = WegItems::get_parts_of_deprecated_inline_command(
            "C:\\Program Files\\node.exe script.js",
        );
        assert_eq!(program, "C:\\Program Files\\node.exe script.js");
        assert_eq!(args, "");
    }

    #[test]
    fn should_handle_command_with_only_quoted_program_and_no_args() {
        let (program, args) =
            WegItems::get_parts_of_deprecated_inline_command("\"C:\\Program Files\\node.exe\"");
        assert_eq!(program, "C:\\Program Files\\node.exe");
        assert_eq!(args, "");
    }

    #[test]
    fn should_preserve_all_spaces_between_arguments() {
        let (program, args) =
            WegItems::get_parts_of_deprecated_inline_command("node  script.js   arg1   arg2");
        assert_eq!(program, "node  script.js   arg1   arg2");
        assert_eq!(args, "");
    }

    #[test]
    fn should_trim_spaces_from_program() {
        let (program, args) = WegItems::get_parts_of_deprecated_inline_command("node    ");
        assert_eq!(program, "node");
        assert_eq!(args, "");
    }

    #[test]
    fn should_handle_complex_quoted_arguments_as_single_string() {
        let (program, args) = WegItems::get_parts_of_deprecated_inline_command(
            "node \"arg with spaces\" 'another arg' --flag=\"value\"",
        );
        assert_eq!(
            program,
            "node \"arg with spaces\" 'another arg' --flag=\"value\""
        );
        assert_eq!(args, "");
    }

    #[test]
    fn should_handle_complex_quoted_arguments_as_single_string_2() {
        let (program, args) = WegItems::get_parts_of_deprecated_inline_command(
            "\"node\" \"arg with spaces\" 'another arg' --flag=\"value\"",
        );
        assert_eq!(program, "node");
        assert_eq!(args, "\"arg with spaces\" 'another arg' --flag=\"value\"");
    }
}
