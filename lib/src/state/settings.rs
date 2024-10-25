/* In this file we use #[serde_alias(SnakeCase)] as backward compatibility from versions below v1.9.8 */

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_alias::serde_alias;

use crate::rect::Rect;

use super::MonitorConfiguration;

// ============== Fancy Toolbar Settings ==============

#[serde_alias(SnakeCase)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct FancyToolbarSettings {
    /// enable or disable the fancy toolbar
    pub enabled: bool,
    /// height of the fancy toolbar
    pub height: u32,
    /// default placeholder for the fancy toolbar
    pub placeholder: String,
    /// hide mode
    pub hide_mode: HideMode,
}

impl Default for FancyToolbarSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            height: 30,
            placeholder: String::from("default.yml"),
            hide_mode: HideMode::Never,
        }
    }
}

// ============== SeelenWeg Settings ==============

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum SeelenWegMode {
    #[serde(rename = "Full-Width")]
    FullWidth,
    #[serde(rename = "Min-Content")]
    MinContent,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum HideMode {
    /// never hide
    Never,
    /// auto-hide always on
    Always,
    /// auto-hide only if is overlaped by the focused window
    #[serde(rename = "On-Overlap")]
    OnOverlap,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum SeelenWegSide {
    Left,
    Right,
    Top,
    Bottom,
}

#[serde_alias(SnakeCase)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct SeelenWegSettings {
    /// enable or disable the seelenweg
    pub enabled: bool,
    /// Dock/Taskbar mode
    pub mode: SeelenWegMode,
    /// When to hide the dock
    pub hide_mode: HideMode,
    /// Dock position
    pub position: SeelenWegSide,
    /// enable or disable separators visibility
    pub visible_separators: bool,
    /// item size in px
    pub size: u32,
    /// zoomed item size in px
    pub zoom_size: u32,
    /// Dock/Taskbar margin in px
    pub margin: u32,
    /// Dock/Taskbar padding in px
    pub padding: u32,
    /// space between items in px
    pub space_between_items: u32,
}

impl Default for SeelenWegSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: SeelenWegMode::MinContent,
            hide_mode: HideMode::OnOverlap,
            position: SeelenWegSide::Bottom,
            visible_separators: true,
            size: 40,
            zoom_size: 70,
            margin: 8,
            padding: 8,
            space_between_items: 8,
        }
    }
}

impl SeelenWegSettings {
    /// total height or width of the dock, depending on the Position
    pub fn total_size(&self) -> u32 {
        self.size + (self.padding * 2) + (self.margin * 2)
    }
}

// ============== Window Manager Settings ==============

#[serde_alias(SnakeCase)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct Border {
    pub enabled: bool,
    pub width: f64,
    pub offset: f64,
}

#[serde_alias(SnakeCase)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct FloatingWindowSettings {
    pub width: f64,
    pub height: f64,
}

#[serde_alias(SnakeCase)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct WindowManagerSettings {
    /// enable or disable the window manager
    pub enabled: bool,
    /// enable or disable auto stacking by category
    pub auto_stacking_by_category: bool,
    /// window manager border
    pub border: Border,
    /// the resize size in % to be used when resizing via cli
    pub resize_delta: f32,
    /// default gap between containers
    pub workspace_gap: u32,
    /// default workspace padding
    pub workspace_padding: u32,
    /// default workspace margin
    #[serde(alias = "global_work_area_offset")]
    pub workspace_margin: Rect,
    /// floating window settings
    pub floating: FloatingWindowSettings,
    /// default layout
    pub default_layout: String,
}

impl Default for Border {
    fn default() -> Self {
        Self {
            enabled: true,
            width: 3.0,
            offset: 0.0,
        }
    }
}

impl Default for FloatingWindowSettings {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 500.0,
        }
    }
}

impl Default for WindowManagerSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_stacking_by_category: true,
            border: Border::default(),
            resize_delta: 10.0,
            workspace_gap: 10,
            workspace_padding: 10,
            workspace_margin: Rect::default(),
            floating: FloatingWindowSettings::default(),
            default_layout: String::from("default.yml"),
        }
    }
}

// ================= Seelen Launcher ================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum SeelenLauncherMonitor {
    Primary,
    #[serde(rename = "Mouse-Over")]
    MouseOver,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct SeelenLauncherRunner {
    pub id: String,
    pub label: String,
    pub program: String,
    pub readonly: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct SeelenLauncherSettings {
    pub enabled: bool,
    pub monitor: SeelenLauncherMonitor,
    pub runners: Vec<SeelenLauncherRunner>,
}

impl Default for SeelenLauncherSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            monitor: SeelenLauncherMonitor::MouseOver,
            runners: vec![
                SeelenLauncherRunner {
                    id: "RUN".to_owned(),
                    label: "t:app_launcher.runners.explorer".to_owned(),
                    program: "explorer.exe".to_owned(),
                    readonly: true,
                },
                SeelenLauncherRunner {
                    id: "CMD".to_owned(),
                    label: "t:app_launcher.runners.cmd".to_owned(),
                    program: "cmd.exe".to_owned(),
                    readonly: true,
                },
            ],
        }
    }
}

impl SeelenLauncherSettings {
    pub fn sanitize(&mut self) {
        let mut dict = HashSet::new();
        self.runners
            .retain(|runner| !runner.program.is_empty() && dict.insert(runner.program.clone()));
        for runner in &mut self.runners {
            if runner.id.is_empty() {
                runner.id = uuid::Uuid::new_v4().to_string();
            }
        }
    }
}

// ================= Seelen Wall ================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SeelenWallWallpaper {
    pub id: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct SeelenWallSettings {
    pub enabled: bool,
    pub backgrounds: Vec<SeelenWallWallpaper>,
    /// update interval in seconds
    pub interval: u64,
}

impl Default for SeelenWallSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            backgrounds: vec![],
            interval: 60,
        }
    }
}

impl SeelenWallSettings {
    pub fn sanitize(&mut self) {
        self.backgrounds.retain(|b| b.path.exists());
    }
}

// ============== Ahk Variables ==============

#[macro_export]
macro_rules! define_struct_and_hashmap {
    (
        $($field:ident),*
    ) => {
        #[serde_alias(SnakeCase)]
        #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
        #[serde(default, rename_all = "camelCase")]
        pub struct AhkVarList {
            $(
                pub $field: AhkVar,
            )*
        }

        impl AhkVarList {
            pub fn as_hash_map(&self) -> HashMap<String, AhkVar> {
                let mut map = HashMap::new();
                $(
                    map.insert(
                        stringify!($field).to_string(),
                        self.$field.clone()
                    );
                )*
                map
            }
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AhkVar {
    pub fancy: String,
    pub ahk: String,
    #[serde(default)]
    pub readonly: bool,
}

impl AhkVar {
    pub fn new(f: &str, ahk: &str) -> Self {
        Self {
            fancy: f.to_string(),
            ahk: ahk.to_string(),
            readonly: false,
        }
    }

    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }
}

define_struct_and_hashmap![
    toggle_launcher,
    reserve_top,
    reserve_bottom,
    reserve_left,
    reserve_right,
    reserve_float,
    reserve_stack,
    focus_top,
    focus_bottom,
    focus_left,
    focus_right,
    focus_latest,
    increase_width,
    decrease_width,
    increase_height,
    decrease_height,
    restore_sizes,
    switch_workspace_0,
    switch_workspace_1,
    switch_workspace_2,
    switch_workspace_3,
    switch_workspace_4,
    switch_workspace_5,
    switch_workspace_6,
    switch_workspace_7,
    switch_workspace_8,
    switch_workspace_9,
    move_to_workspace_0,
    move_to_workspace_1,
    move_to_workspace_2,
    move_to_workspace_3,
    move_to_workspace_4,
    move_to_workspace_5,
    move_to_workspace_6,
    move_to_workspace_7,
    move_to_workspace_8,
    move_to_workspace_9,
    send_to_workspace_0,
    send_to_workspace_1,
    send_to_workspace_2,
    send_to_workspace_3,
    send_to_workspace_4,
    send_to_workspace_5,
    send_to_workspace_6,
    send_to_workspace_7,
    send_to_workspace_8,
    send_to_workspace_9,
    misc_open_settings,
    misc_toggle_lock_tracing,
    misc_toggle_win_event_tracing
];

impl Default for AhkVarList {
    fn default() -> Self {
        Self {
            // launcher
            toggle_launcher: AhkVar::new("Win + Space", "LWin & Space").readonly(),
            // wm
            reserve_top: AhkVar::new("Win + Shift + I", "#+i"),
            reserve_bottom: AhkVar::new("Win + Shift + K", "#+k"),
            reserve_left: AhkVar::new("Win + Shift + J", "#+j"),
            reserve_right: AhkVar::new("Win + Shift + L", "#+l"),
            reserve_float: AhkVar::new("Win + Shift + U", "#+u"),
            reserve_stack: AhkVar::new("Win + Shift + O", "#+o"),
            focus_top: AhkVar::new("Win + Shift + W", "#+w"),
            focus_bottom: AhkVar::new("Win + Shift + S", "#+s"),
            focus_left: AhkVar::new("Win + Shift + A", "#+a"),
            focus_right: AhkVar::new("Win + Shift + D", "#+d"),
            focus_latest: AhkVar::new("Win + Shift + E", "#+e"),
            increase_width: AhkVar::new("Win + Alt + =", "#!="),
            decrease_width: AhkVar::new("Win + Alt + -", "#!-"),
            increase_height: AhkVar::new("Win + Shift + =", "#+="),
            decrease_height: AhkVar::new("Win + Shift + -", "#+-"),
            restore_sizes: AhkVar::new("Win + Alt + 0", "#!0"),
            // virtual desktops
            switch_workspace_0: AhkVar::new("Alt + 1", "!1"),
            switch_workspace_1: AhkVar::new("Alt + 2", "!2"),
            switch_workspace_2: AhkVar::new("Alt + 3", "!3"),
            switch_workspace_3: AhkVar::new("Alt + 4", "!4"),
            switch_workspace_4: AhkVar::new("Alt + 5", "!5"),
            switch_workspace_5: AhkVar::new("Alt + 6", "!6"),
            switch_workspace_6: AhkVar::new("Alt + 7", "!7"),
            switch_workspace_7: AhkVar::new("Alt + 8", "!8"),
            switch_workspace_8: AhkVar::new("Alt + 9", "!9"),
            switch_workspace_9: AhkVar::new("Alt + 0", "!0"),
            move_to_workspace_0: AhkVar::new("Alt + Shift + 1", "!+1"),
            move_to_workspace_1: AhkVar::new("Alt + Shift + 2", "!+2"),
            move_to_workspace_2: AhkVar::new("Alt + Shift + 3", "!+3"),
            move_to_workspace_3: AhkVar::new("Alt + Shift + 4", "!+4"),
            move_to_workspace_4: AhkVar::new("Alt + Shift + 5", "!+5"),
            move_to_workspace_5: AhkVar::new("Alt + Shift + 6", "!+6"),
            move_to_workspace_6: AhkVar::new("Alt + Shift + 7", "!+7"),
            move_to_workspace_7: AhkVar::new("Alt + Shift + 8", "!+8"),
            move_to_workspace_8: AhkVar::new("Alt + Shift + 9", "!+9"),
            move_to_workspace_9: AhkVar::new("Alt + Shift + 0", "!+0"),
            send_to_workspace_0: AhkVar::new("Win + Shift + 1", "#+1"),
            send_to_workspace_1: AhkVar::new("Win + Shift + 2", "#+2"),
            send_to_workspace_2: AhkVar::new("Win + Shift + 3", "#+3"),
            send_to_workspace_3: AhkVar::new("Win + Shift + 4", "#+4"),
            send_to_workspace_4: AhkVar::new("Win + Shift + 5", "#+5"),
            send_to_workspace_5: AhkVar::new("Win + Shift + 6", "#+6"),
            send_to_workspace_6: AhkVar::new("Win + Shift + 7", "#+7"),
            send_to_workspace_7: AhkVar::new("Win + Shift + 8", "#+8"),
            send_to_workspace_8: AhkVar::new("Win + Shift + 9", "#+9"),
            send_to_workspace_9: AhkVar::new("Win + Shift + 0", "#+0"),
            // miscellaneous
            misc_open_settings: AhkVar::new("Win + K", "#k").readonly(),
            misc_toggle_lock_tracing: AhkVar::new("Ctrl + Win + Alt + T", "^#!t").readonly(),
            misc_toggle_win_event_tracing: AhkVar::new("Ctrl + Win + Alt + L", "^#!l").readonly(),
        }
    }
}

// ========================== Seelen Updates ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum UpdateChannel {
    Release,
    Beta,
    Nightly,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdaterSettings {
    pub channel: UpdateChannel,
}

impl Default for UpdaterSettings {
    fn default() -> Self {
        Self {
            channel: UpdateChannel::Release,
        }
    }
}

// ======================== Final Settings Struct ===============================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum VirtualDesktopStrategy {
    Native,
    Seelen,
}

#[serde_alias(SnakeCase)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    /// fancy toolbar config
    pub fancy_toolbar: FancyToolbarSettings,
    /// seelenweg (dock/taskbar) config
    pub seelenweg: SeelenWegSettings,
    /// window manager config
    pub window_manager: WindowManagerSettings,
    /// background and virtual desktops config
    pub wall: SeelenWallSettings,
    /// App launcher settings
    pub launcher: SeelenLauncherSettings,
    /// list of monitors
    pub monitors: Vec<MonitorConfiguration>,
    /// enable or disable ahk
    pub ahk_enabled: bool,
    /// ahk variables
    pub ahk_variables: AhkVarList,
    /// list of selected themes
    #[serde(alias = "selected_theme")]
    pub selected_themes: Vec<String>,
    /// list of selected icon packs
    pub icon_packs: Vec<String>,
    /// enable or disable dev tools tab in settings
    pub dev_tools: bool,
    /// language to use, if null the system locale is used
    pub language: Option<String>,
    /// MomentJS date format
    pub date_format: String,
    /// what virtual desktop implementation will be used, in case Native is not available we use Seelen
    pub virtual_desktop_strategy: VirtualDesktopStrategy,
    /// Updater Settings
    pub updater: UpdaterSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ahk_enabled: true,
            selected_themes: vec!["default".to_string()],
            icon_packs: vec!["system".to_string()],
            monitors: vec![MonitorConfiguration::default()],
            fancy_toolbar: FancyToolbarSettings::default(),
            seelenweg: SeelenWegSettings::default(),
            window_manager: WindowManagerSettings::default(),
            wall: SeelenWallSettings::default(),
            launcher: SeelenLauncherSettings::default(),
            ahk_variables: AhkVarList::default(),
            dev_tools: false,
            language: Some(Self::get_system_language()),
            date_format: "ddd D MMM, hh:mm A".to_owned(),
            virtual_desktop_strategy: VirtualDesktopStrategy::Native,
            updater: UpdaterSettings::default(),
        }
    }
}

impl Settings {
    pub fn get_locale() -> Option<String> {
        sys_locale::get_locale()
    }

    pub fn get_system_language() -> String {
        match sys_locale::get_locale() {
            Some(l) => l.split('-').next().unwrap_or("en").to_string(),
            None => "en".to_string(),
        }
    }

    pub fn sanitize(&mut self) {
        self.launcher.sanitize();
        self.wall.sanitize();

        if self.language.is_none() {
            self.language = Some(Self::get_system_language());
        }

        let default_theme = "default".to_owned();
        if !self.selected_themes.contains(&default_theme) {
            self.selected_themes.insert(0, default_theme);
        }

        let default_icon_pack = "system".to_owned();
        if !self.icon_packs.contains(&default_icon_pack) {
            self.icon_packs.insert(0, default_icon_pack);
        }
    }
}
