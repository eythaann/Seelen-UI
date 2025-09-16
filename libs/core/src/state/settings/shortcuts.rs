use std::collections::HashSet;

use uuid::Uuid;

macro_rules! define_hotkey_actions {
    (
        $(
            $field:ident$(($arg:ty))? $(= [$($key:literal),*])?
        ),*
    ) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, TS)]
        #[serde(tag = "name", content = "arg", rename_all = "snake_case")]
        pub enum SluHotkeyAction {
            $(
                $field$(($arg))?,
            )*
        }

        impl SluShortcutsSettings {
            fn _default_shortcuts() -> Vec<SluHotkey> {
                vec![
                    $($(
                        SluHotkey::new(SluHotkeyAction::$field).keys([$($key),*]),
                    )?)*
                ]
            }
        }
    };
}

define_hotkey_actions! {
    ToggleLauncher = ["Win", "S"],
    // tiling-wm state
    PauseTiling = ["Win", "P"],
    ToggleFloat = ["Win", "F"],
    ToggleMonocle = ["Win", "M"],
    CycleStackNext = ["Win", "Alt", "Right"],
    CycleStackPrev = ["Win", "Alt", "Left"],
    // tiling-wm reservation
    ReserveTop = ["Win", "Shift", "I"],
    ReserveBottom = ["Win", "Shift", "K"],
    ReserveLeft = ["Win", "Shift", "J"],
    ReserveRight = ["Win", "Shift", "L"],
    ReserveFloat = ["Win", "Shift", "U"],
    ReserveStack = ["Win", "Shift", "O"],
    // tiling-wm focus change actions
    FocusTop = ["Alt", "I"],
    FocusBottom = ["Alt", "K"],
    FocusLeft = ["Alt", "J"],
    FocusRight = ["Alt", "L"],
    // wm focused window sizing
    IncreaseWidth = ["Win", "Alt", "="],
    DecreaseWidth = ["Win", "Alt", "-"],
    IncreaseHeight = ["Win", "Ctrl", "="],
    DecreaseHeight = ["Win", "Ctrl", "-"],
    RestoreSizes = ["Win", "Alt", "0"],
    // wm focused window positioning
    MoveWindowUp = ["Shift", "Alt", "I"],
    MoveWindowDown = ["Shift", "Alt", "K"],
    MoveWindowLeft = ["Shift", "Alt", "J"],
    MoveWindowRight = ["Shift", "Alt", "L"],
    // weg
    StartWegApp(usize),
    // virtual desktops
    SwitchWorkspace(usize),
    MoveToWorkspace(usize),
    SendToWorkspace(usize),
    SwitchToNextWorkspace = ["Ctrl", "Win", "Right"],
    SwitchToPreviousWorkspace = ["Ctrl", "Win", "Left"],
    CreateNewWorkspace = ["Ctrl", "Win", "D"],
    DestroyCurrentWorkspace = ["Ctrl", "Win", "F4"],
    // wallpaper manager
    CycleWallpaperNext = ["Ctrl", "Win", "Up"],
    CycleWallpaperPrev = ["Ctrl", "Win", "Down"],
    // misc
    MiscOpenSettings = ["Win", "K"],
    MiscForceRestart = ["Ctrl", "Win", "Alt", "R"],
    MiscToggleLockTracing,
    MiscToggleWinEventTracing
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
pub struct SluHotkey {
    pub id: Uuid,
    pub action: SluHotkeyAction,
    pub keys: Vec<String>,
    #[serde(default)]
    pub readonly: bool,
}

impl SluHotkey {
    pub fn new(action: SluHotkeyAction) -> Self {
        Self {
            id: Uuid::new_v4(),
            action,
            keys: vec![],
            readonly: false,
        }
    }

    pub fn keys<'a, T, I>(mut self, keys: I) -> Self
    where
        T: AsRef<str> + 'a,
        I: IntoIterator<Item = T>,
    {
        self.keys = keys.into_iter().map(|k| k.as_ref().to_string()).collect();
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
        self.app_commands
            .retain(|h| seen_ids.insert(h.id) && !h.keys.is_empty());
    }

    pub fn get_mut(&mut self, action: SluHotkeyAction) -> Option<&mut SluHotkey> {
        self.app_commands.iter_mut().find(|h| h.action == action)
    }

    pub fn default_shortcuts() -> Self {
        let mut shorcuts = Self::_default_shortcuts();

        for i in 0..10 {
            let digit_key = if i == 9 {
                String::from("0")
            } else {
                format!("{}", i + 1)
            };

            shorcuts.push(
                SluHotkey::new(SluHotkeyAction::StartWegApp(i)).keys(["Win", digit_key.as_str()]),
            );

            shorcuts.push(
                SluHotkey::new(SluHotkeyAction::SwitchWorkspace(i))
                    .keys(["Alt", digit_key.as_str()]),
            );

            shorcuts.push(SluHotkey::new(SluHotkeyAction::MoveToWorkspace(i)).keys([
                "Alt",
                "Shift",
                digit_key.as_str(),
            ]));

            shorcuts.push(SluHotkey::new(SluHotkeyAction::SendToWorkspace(i)).keys([
                "Win",
                "Shift",
                digit_key.as_str(),
            ]));
        }

        let mut defaults = Self {
            enabled: true,
            app_commands: shorcuts,
        };

        if let Some(h) = defaults.get_mut(SluHotkeyAction::MiscForceRestart) {
            h.readonly = true;
        }

        defaults
    }
}

impl Default for SluShortcutsSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            app_commands: Vec::new(),
        }
    }
}
