use std::collections::HashSet;

use uuid::Uuid;

use crate::resource::WidgetId;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "name", rename_all = "snake_case")]
pub enum SluHotkeyAction {
    ToggleAppsMenu,
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
    /// This will be true for hotkeys intended to override system hotkeys.
    #[serde(default)]
    pub system: bool,
    /// If present this shortcut will be only available if the widget is enabled.
    #[serde(default)]
    pub attached_to: Option<WidgetId>,
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
            attached_to: None,
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

    pub fn attached_to(mut self, widget_id: impl Into<WidgetId>) -> Self {
        self.attached_to = Some(widget_id.into());
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
                .system()
                .attached_to("@seelen/weg"),
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

        let wm = "@seelen/window-manager";

        vec![
            SluHotkey::new(ToggleAppsMenu, ["Win"]).attached_to("@seelen/apps-menu"),
            // Task switching and viewer
            SluHotkey::new(
                TaskNext {
                    select_on_key_up: true,
                },
                ["Alt", "Tab"],
            )
            .system()
            .attached_to("@seelen/task-switcher"),
            SluHotkey::new(
                TaskPrev {
                    select_on_key_up: true,
                },
                ["Alt", "Shift", "Tab"],
            )
            .system()
            .attached_to("@seelen/task-switcher"),
            SluHotkey::new(
                TaskNext {
                    select_on_key_up: false,
                },
                ["Alt", "Ctrl", "Tab"],
            )
            .system()
            .attached_to("@seelen/task-switcher"),
            SluHotkey::new(
                TaskPrev {
                    select_on_key_up: false,
                },
                ["Alt", "Ctrl", "Shift", "Tab"],
            )
            .system()
            .attached_to("@seelen/task-switcher"),
            // tiling window manager
            SluHotkey::new(PauseTiling, ["Win", "P"]).attached_to(wm),
            SluHotkey::new(ToggleFloat, ["Win", "F"]).attached_to(wm),
            SluHotkey::new(ToggleMonocle, ["Win", "M"]).attached_to(wm),
            //
            SluHotkey::new(CycleStackNext, ["Win", "Alt", "Right"]).attached_to(wm),
            SluHotkey::new(CycleStackPrev, ["Win", "Alt", "Left"]).attached_to(wm),
            //
            SluHotkey::new(ReserveTop, ["Win", "Shift", "I"]).attached_to(wm),
            SluHotkey::new(ReserveBottom, ["Win", "Shift", "K"]).attached_to(wm),
            SluHotkey::new(ReserveLeft, ["Win", "Shift", "J"]).attached_to(wm),
            SluHotkey::new(ReserveRight, ["Win", "Shift", "L"]).attached_to(wm),
            SluHotkey::new(ReserveFloat, ["Win", "Shift", "U"]).attached_to(wm),
            SluHotkey::new(ReserveStack, ["Win", "Shift", "O"]).attached_to(wm),
            //
            SluHotkey::new(FocusTop, ["Alt", "I"]).attached_to(wm),
            SluHotkey::new(FocusBottom, ["Alt", "K"]).attached_to(wm),
            SluHotkey::new(FocusLeft, ["Alt", "J"]).attached_to(wm),
            SluHotkey::new(FocusRight, ["Alt", "L"]).attached_to(wm),
            //
            SluHotkey::new(IncreaseWidth, ["Win", "Alt", "="]).attached_to(wm),
            SluHotkey::new(DecreaseWidth, ["Win", "Alt", "-"]).attached_to(wm),
            SluHotkey::new(IncreaseHeight, ["Win", "Ctrl", "="]).attached_to(wm),
            SluHotkey::new(DecreaseHeight, ["Win", "Ctrl", "-"]).attached_to(wm),
            SluHotkey::new(RestoreSizes, ["Win", "Alt", "0"]).attached_to(wm),
            //
            SluHotkey::new(MoveWindowUp, ["Shift", "Alt", "I"]).attached_to(wm),
            SluHotkey::new(MoveWindowDown, ["Shift", "Alt", "K"]).attached_to(wm),
            SluHotkey::new(MoveWindowLeft, ["Shift", "Alt", "J"]).attached_to(wm),
            SluHotkey::new(MoveWindowRight, ["Shift", "Alt", "L"]).attached_to(wm),
            // virtual desktop
            SluHotkey::new(SwitchToNextWorkspace, ["Ctrl", "Win", "Right"]).system(),
            SluHotkey::new(SwitchToPreviousWorkspace, ["Ctrl", "Win", "Left"]).system(),
            SluHotkey::new(CreateNewWorkspace, ["Ctrl", "Win", "D"]).system(),
            SluHotkey::new(DestroyCurrentWorkspace, ["Ctrl", "Win", "F4"]).system(),
            SluHotkey::new(ToggleWorkspacesView, ["Win", "Tab"])
                .system()
                .attached_to("@seelen/workspaces-viewer"),
            // wallpaper manager
            SluHotkey::new(CycleWallpaperNext, ["Ctrl", "Win", "Up"]),
            SluHotkey::new(CycleWallpaperPrev, ["Ctrl", "Win", "Down"]),
            // misc
            SluHotkey::new(MiscOpenSettings, ["Win", "K"]),
            SluHotkey::new(MiscForceRestart, ["Ctrl", "Win", "Alt", "R"]).readonly(),
            SluHotkey::new(MiscForceQuit, ["Ctrl", "Win", "Alt", "K"]).readonly(),
        ]
    }
}
