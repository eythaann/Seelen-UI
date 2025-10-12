use std::collections::HashSet;

use uuid::Uuid;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "name", rename_all = "snake_case")]
pub enum SluHotkeyAction {
    ToggleLauncher,
    ToggleWorkspacesView,
    // ==========================
    TaskNext {
        select_on_key_up: bool,
    },
    TaskPrev {
        select_on_key_up: bool,
    },
    // ==========================
    PauseTiling,
    ToggleFloat,
    ToggleMonocle,
    CycleStackNext,
    CycleStackPrev,
    ReserveTop,
    ReserveBottom,
    ReserveLeft,
    ReserveRight,
    ReserveFloat,
    ReserveStack,
    FocusTop,
    FocusBottom,
    FocusLeft,
    FocusRight,
    IncreaseWidth,
    DecreaseWidth,
    IncreaseHeight,
    DecreaseHeight,
    RestoreSizes,
    MoveWindowUp,
    MoveWindowDown,
    MoveWindowLeft,
    MoveWindowRight,
    // ==========================
    StartWegApp {
        #[serde(alias = "arg")]
        index: usize,
    },
    // ==========================
    SwitchWorkspace {
        #[serde(alias = "arg")]
        index: usize,
    },
    MoveToWorkspace {
        #[serde(alias = "arg")]
        index: usize,
    },
    SendToWorkspace {
        #[serde(alias = "arg")]
        index: usize,
    },
    SwitchToNextWorkspace,
    SwitchToPreviousWorkspace,
    CreateNewWorkspace,
    DestroyCurrentWorkspace,
    // ==========================
    CycleWallpaperNext,
    CycleWallpaperPrev,
    // ==========================
    MiscOpenSettings,
    MiscForceRestart,
    MiscForceQuit,
    MiscToggleLockTracing,
    MiscToggleWinEventTracing,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
pub struct SluHotkey {
    pub id: Uuid,
    pub action: SluHotkeyAction,
    pub keys: Vec<String>,
    #[serde(default)]
    pub readonly: bool,
    /// This will be true for hotkeys intended to override system hotkeys
    #[serde(default)]
    pub system: bool,
}

impl SluHotkey {
    pub fn new<'a, T, I>(action: SluHotkeyAction, keys: I) -> Self
    where
        T: AsRef<str> + 'a,
        I: IntoIterator<Item = T>,
    {
        Self {
            id: Uuid::new_v4(),
            action,
            keys: keys.into_iter().map(|k| k.as_ref().to_string()).collect(),
            readonly: false,
            system: false,
        }
    }

    pub fn system(mut self) -> Self {
        self.system = true;
        self
    }

    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct SluShortcutsSettings {
    pub enabled: bool,
    pub app_commands: Vec<SluHotkey>,
}

impl Default for SluShortcutsSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            app_commands: Vec::new(),
        }
    }
}

impl SluShortcutsSettings {
    pub fn contains_action(&self, action: SluHotkeyAction) -> bool {
        self.app_commands.iter().any(|h| h.action == action)
    }

    pub fn sanitize(&mut self) {
        let defaults = Self::default_shortcuts();
        for hotkey in defaults.app_commands {
            // add missing hotkeys from defaults
            if !self.contains_action(hotkey.action) {
                self.app_commands.push(hotkey);
            }
        }

        let mut seen_ids = HashSet::new();
        self.app_commands.retain(|h| {
            seen_ids.insert(h.id) && !h.keys.is_empty() && h.action != SluHotkeyAction::Unknown
        });
    }

    pub fn get_mut(&mut self, action: SluHotkeyAction) -> Option<&mut SluHotkey> {
        self.app_commands.iter_mut().find(|h| h.action == action)
    }

    pub fn default_shortcuts() -> Self {
        let mut shorcuts = Self::_default_shortcuts();

        for index in 0..10 {
            let digit_key = if index == 9 {
                String::from("0")
            } else {
                format!("{}", index + 1)
            };

            shorcuts.push(
                SluHotkey::new(
                    SluHotkeyAction::StartWegApp { index },
                    ["Win", digit_key.as_str()],
                )
                .system(),
            );

            shorcuts.push(SluHotkey::new(
                SluHotkeyAction::SwitchWorkspace { index },
                ["Alt", digit_key.as_str()],
            ));

            shorcuts.push(SluHotkey::new(
                SluHotkeyAction::MoveToWorkspace { index },
                ["Alt", "Shift", digit_key.as_str()],
            ));

            shorcuts.push(SluHotkey::new(
                SluHotkeyAction::SendToWorkspace { index },
                ["Win", "Shift", digit_key.as_str()],
            ));
        }

        Self {
            enabled: true,
            app_commands: shorcuts,
        }
    }

    fn _default_shortcuts() -> Vec<SluHotkey> {
        use SluHotkeyAction::*;

        vec![
            SluHotkey::new(ToggleLauncher, ["Win", "S"]),
            SluHotkey::new(ToggleWorkspacesView, ["Win", "Tab"]).system(),
            // ==========================================
            SluHotkey::new(
                TaskNext {
                    select_on_key_up: true,
                },
                ["Alt", "Tab"],
            )
            .system(),
            SluHotkey::new(
                TaskPrev {
                    select_on_key_up: true,
                },
                ["Alt", "Shift", "Tab"],
            )
            .system(),
            SluHotkey::new(
                TaskNext {
                    select_on_key_up: false,
                },
                ["Alt", "Ctrl", "Tab"],
            )
            .system(),
            SluHotkey::new(
                TaskPrev {
                    select_on_key_up: false,
                },
                ["Alt", "Ctrl", "Shift", "Tab"],
            )
            .system(),
            // ==========================================
            SluHotkey::new(PauseTiling, ["Win", "P"]),
            SluHotkey::new(ToggleFloat, ["Win", "F"]),
            SluHotkey::new(ToggleMonocle, ["Win", "M"]),
            SluHotkey::new(CycleStackNext, ["Win", "Alt", "Right"]),
            SluHotkey::new(CycleStackPrev, ["Win", "Alt", "Left"]),
            SluHotkey::new(ReserveTop, ["Win", "Shift", "I"]),
            SluHotkey::new(ReserveBottom, ["Win", "Shift", "K"]),
            SluHotkey::new(ReserveLeft, ["Win", "Shift", "J"]),
            SluHotkey::new(ReserveRight, ["Win", "Shift", "L"]),
            SluHotkey::new(ReserveFloat, ["Win", "Shift", "U"]),
            SluHotkey::new(ReserveStack, ["Win", "Shift", "O"]),
            SluHotkey::new(FocusTop, ["Alt", "I"]),
            SluHotkey::new(FocusBottom, ["Alt", "K"]),
            SluHotkey::new(FocusLeft, ["Alt", "J"]),
            SluHotkey::new(FocusRight, ["Alt", "L"]),
            SluHotkey::new(IncreaseWidth, ["Win", "Alt", "="]),
            SluHotkey::new(DecreaseWidth, ["Win", "Alt", "-"]),
            SluHotkey::new(IncreaseHeight, ["Win", "Ctrl", "="]),
            SluHotkey::new(DecreaseHeight, ["Win", "Ctrl", "-"]),
            SluHotkey::new(RestoreSizes, ["Win", "Alt", "0"]),
            SluHotkey::new(MoveWindowUp, ["Shift", "Alt", "I"]),
            SluHotkey::new(MoveWindowDown, ["Shift", "Alt", "K"]),
            SluHotkey::new(MoveWindowLeft, ["Shift", "Alt", "J"]),
            SluHotkey::new(MoveWindowRight, ["Shift", "Alt", "L"]),
            SluHotkey::new(SwitchToNextWorkspace, ["Ctrl", "Win", "Right"]).system(),
            SluHotkey::new(SwitchToPreviousWorkspace, ["Ctrl", "Win", "Left"]).system(),
            SluHotkey::new(CreateNewWorkspace, ["Ctrl", "Win", "D"]).system(),
            SluHotkey::new(DestroyCurrentWorkspace, ["Ctrl", "Win", "F4"]).system(),
            SluHotkey::new(CycleWallpaperNext, ["Ctrl", "Win", "Up"]),
            SluHotkey::new(CycleWallpaperPrev, ["Ctrl", "Win", "Down"]),
            SluHotkey::new(MiscOpenSettings, ["Win", "K"]),
            SluHotkey::new(MiscForceRestart, ["Ctrl", "Win", "Alt", "R"]).readonly(),
            SluHotkey::new(MiscForceQuit, ["Ctrl", "Win", "Alt", "K"]).readonly(),
        ]
    }
}
