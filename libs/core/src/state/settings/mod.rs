/* In this file we use #[serde_alias(SnakeCase)] as backward compatibility from versions below v1.9.8 */
pub mod by_monitor;
pub mod by_theme;
pub mod by_wallpaper;
pub mod by_widget;
pub mod shortcuts;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_alias::serde_alias;
use ts_rs::TS;

use crate::resource::WidgetId;
use crate::system_state::MonitorId;
use crate::{
    error::Result,
    rect::Rect,
    resource::{IconPackId, PluginId, ThemeId, WallpaperId},
    state::{
        by_monitor::MonitorConfiguration, by_theme::ThemeSettings,
        by_wallpaper::WallpaperInstanceSettings, by_widget::SettingsByWidget,
        shortcuts::SluShortcutsSettings,
    },
};

// ============== Fancy Toolbar Settings ==============

#[serde_alias(SnakeCase)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct FancyToolbarSettings {
    /// enable or disable the fancy toolbar
    pub enabled: bool,
    /// height of the fancy toolbar
    pub height: u32,
    /// position of the toolbar
    pub position: FancyToolbarSide,
    /// hide mode
    pub hide_mode: HideMode,
    /// delay to show the toolbar on Mouse Hover in milliseconds
    pub delay_to_show: u32,
    /// delay to hide the toolbar on Mouse Leave in milliseconds
    pub delay_to_hide: u32,
}

impl Default for FancyToolbarSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            height: 30,
            position: FancyToolbarSide::Top,
            hide_mode: HideMode::Never,
            delay_to_show: 100,
            delay_to_hide: 800,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum FancyToolbarSide {
    Top,
    Bottom,
}

// ============== SeelenWeg Settings ==============

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum SeelenWegMode {
    #[serde(alias = "Full-Width")]
    FullWidth,
    #[serde(alias = "Min-Content")]
    MinContent,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum WegTemporalItemsVisibility {
    All,
    OnMonitor,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum WegPinnedItemsVisibility {
    Always,
    WhenPrimary,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum HideMode {
    /// never hide
    Never,
    /// auto-hide always on
    Always,
    /// auto-hide only if is overlaped by the focused window
    #[serde(alias = "On-Overlap")]
    OnOverlap,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum SeelenWegSide {
    Left,
    Right,
    Top,
    Bottom,
}

#[serde_alias(SnakeCase)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct SeelenWegSettings {
    /// enable or disable the seelenweg
    pub enabled: bool,
    /// Dock/Taskbar mode
    pub mode: SeelenWegMode,
    /// When to hide the dock
    pub hide_mode: HideMode,
    /// Which temporal items to show on the dock instance (this can be overridden per monitor)
    pub temporal_items_visibility: WegTemporalItemsVisibility,
    /// Determines is the pinned item should be shown or not (this can be overridden per monitor).
    pub pinned_items_visibility: WegPinnedItemsVisibility,
    /// Dock position
    pub position: SeelenWegSide,
    /// Decides whether the application hoover should generate thumbnails or just list the names instead
    pub thumbnail_generation_enabled: bool,
    /// enable or disable the instance counter visibility on weg instance
    pub show_instance_counter: bool,
    /// enable or disable the window title visibility for opened apps
    pub show_window_title: bool,
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
    /// delay to show the toolbar on Mouse Hover in milliseconds
    pub delay_to_show: u32,
    /// delay to hide the toolbar on Mouse Leave in milliseconds
    pub delay_to_hide: u32,
    /// show end task button on context menu (needs developer mode enabled)
    pub show_end_task: bool,
}

impl Default for SeelenWegSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: SeelenWegMode::MinContent,
            hide_mode: HideMode::OnOverlap,
            position: SeelenWegSide::Bottom,
            thumbnail_generation_enabled: true,
            visible_separators: true,
            show_instance_counter: true,
            show_window_title: false,
            temporal_items_visibility: WegTemporalItemsVisibility::All,
            pinned_items_visibility: WegPinnedItemsVisibility::Always,
            size: 40,
            zoom_size: 70,
            margin: 8,
            padding: 8,
            space_between_items: 8,
            delay_to_show: 100,
            delay_to_hide: 800,
            show_end_task: false,
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct WindowManagerSettings {
    /// enable or disable the tiling window manager
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
    pub workspace_margin: Rect,
    /// floating window settings
    pub floating: FloatingWindowSettings,
    /// default layout
    pub default_layout: PluginId,
    /// window manager animations
    pub animations: WmAnimations,
    /// window manager drag behavior
    pub drag_behavior: WmDragBehavior,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum WmDragBehavior {
    /// While dragging the windows on the layout will be sorted.
    Sort,
    /// On drag end the dragged and the overlaped will be swapped.
    Swap,
}

#[serde_alias(SnakeCase)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct Border {
    pub enabled: bool,
    pub width: f64,
    pub offset: f64,
}

#[serde_alias(SnakeCase)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct FloatingWindowSettings {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct WmAnimations {
    pub enabled: bool,
    pub duration_ms: u64,
    pub ease_function: String,
}

impl Default for WmAnimations {
    fn default() -> Self {
        Self {
            enabled: true,
            duration_ms: 200,
            ease_function: "EaseOut".into(),
        }
    }
}

impl Default for Border {
    fn default() -> Self {
        Self {
            enabled: true,
            offset: 0.0,
            width: 3.0,
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
            default_layout: "@default/wm-bspwm".into(),
            animations: WmAnimations::default(),
            drag_behavior: WmDragBehavior::Sort,
        }
    }
}

// ================= Seelen Launcher ================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum SeelenLauncherMonitor {
    Primary,
    #[serde(alias = "Mouse-Over")]
    MouseOver,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct SeelenLauncherRunner {
    pub id: String,
    pub label: String,
    pub program: String,
    pub readonly: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
pub struct LauncherHistory(HashMap<String, Vec<String>>);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct SeelenWallSettings {
    pub enabled: bool,
    pub backgrounds_v2: Vec<WallpaperId>,
    /// update interval in seconds
    pub interval: u32,
    /// randomize order
    pub randomize: bool,
}

impl Default for SeelenWallSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            backgrounds_v2: vec![],
            interval: 60,
            randomize: false,
        }
    }
}

// ========================== Seelen Updates ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum UpdateChannel {
    Release,
    Beta,
    Nightly,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
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
#[serde_alias(SnakeCase)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct Settings {
    /// @deprecated since v2.1.0, will be removed in v3.0.0
    #[ts(skip)]
    #[serde(skip_serializing)]
    fancy_toolbar: Option<FancyToolbarSettings>,
    ///@deprecated since v2.1.0, will be removed in v3.0.0
    #[ts(skip)]
    #[serde(skip_serializing)]
    seelenweg: Option<SeelenWegSettings>,
    /// @deprecated since v2.1.0, will be removed in v3.0.0
    #[ts(skip)]
    #[serde(skip_serializing)]
    window_manager: Option<WindowManagerSettings>,
    /// @deprecated since v2.1.0, will be removed in v3.0.0
    #[ts(skip)]
    #[serde(skip_serializing)]
    wall: Option<SeelenWallSettings>,
    /// @deprecated since v2.1.0, will be removed in v3.0.0
    #[ts(skip)]
    #[serde(skip_serializing)]
    launcher: Option<SeelenLauncherSettings>,
    /// list of monitors and their configurations
    pub monitors_v3: HashMap<MonitorId, MonitorConfiguration>,
    /// app shortcuts settings
    pub shortcuts: SluShortcutsSettings,
    /// list of selected themes as filename as backguard compatibility for versions before v2.3.8, will be removed in v3
    #[serde(alias = "selectedThemes")]
    pub old_active_themes: Vec<String>,
    /// list of selected themes
    pub active_themes: Vec<ThemeId>,
    /// list of selected icon packs
    pub active_icon_packs: Vec<IconPackId>,
    /// enable or disable dev tools tab in settings
    pub dev_tools: bool,
    /// discord rich presence
    pub drpc: bool,
    /// language to use, if null the system locale is used
    pub language: Option<String>,
    /// MomentJS date format
    pub date_format: String,
    /// Updater Settings
    pub updater: UpdaterSettings,
    /// Custom settings for widgets
    pub by_widget: SettingsByWidget,
    /// Custom variables for themes by theme id
    /// ### example
    /// ```json
    /// {
    ///     "@username/themeName": {
    ///         "--css-variable-name": "123px",
    ///         "--css-variable-name2": "#aabbccaa",
    ///     }
    /// }
    /// ```
    pub by_theme: HashMap<ThemeId, ThemeSettings>,
    /// settings for each background
    pub by_wallpaper: HashMap<WallpaperId, WallpaperInstanceSettings>,
    /// Performance options
    pub performance_mode: PerformanceModeSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fancy_toolbar: None,
            seelenweg: None,
            window_manager: None,
            wall: None,
            launcher: None,
            // ---
            performance_mode: PerformanceModeSettings::default(),
            shortcuts: SluShortcutsSettings::default(),
            drpc: false,
            old_active_themes: Vec::new(),
            active_themes: vec!["@default/theme".into()],
            active_icon_packs: vec!["@system/icon-pack".into()],
            monitors_v3: HashMap::new(),
            dev_tools: false,
            language: Some(Self::get_system_language()),
            date_format: "ddd D MMM, hh:mm A".to_owned(),
            updater: UpdaterSettings::default(),
            by_widget: SettingsByWidget::default(),
            by_theme: HashMap::new(),
            by_wallpaper: HashMap::new(),
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

    /// Migrate old settings (before v2.1.0) (will be removed in v3.0.0)
    pub fn migrate(&mut self) -> Result<()> {
        let dict = &mut self.by_widget;
        if let Some(tb) = self.fancy_toolbar.take() {
            dict.fancy_toolbar = tb;
        }
        if let Some(weg) = self.seelenweg.take() {
            dict.weg = weg;
        }
        if let Some(wm) = self.window_manager.take() {
            dict.wm = wm;
        }
        if let Some(wall) = self.wall.take() {
            dict.wall = wall;
        }
        if let Some(launcher) = self.launcher.take() {
            dict.launcher = launcher;
        }
        Ok(())
    }

    pub fn dedup_themes(&mut self) {
        let mut seen = HashSet::new();
        self.active_themes.retain(|x| seen.insert(x.clone())); // dedup
    }

    pub fn dedup_icon_packs(&mut self) {
        let mut seen = HashSet::new();
        self.active_icon_packs.retain(|x| seen.insert(x.clone())); // dedup
    }

    pub fn sanitize(&mut self) -> Result<()> {
        self.by_widget.launcher.sanitize();

        if self.language.is_none() {
            self.language = Some(Self::get_system_language());
        }

        // ensure base is always selected
        self.active_themes.insert(0, "@default/theme".into());
        self.dedup_themes();
        // ensure base is always selected
        self.active_icon_packs.insert(0, "@system/icon-pack".into());
        self.dedup_icon_packs();

        self.shortcuts.sanitize();
        Ok(())
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let mut settings: Self = {
            let file = File::open(path)?;
            file.lock_shared()?;
            serde_json::from_reader(&file)?
        };

        // Load shortcuts from sibling file if it exists
        if let (Some(parent), Some(stem)) = (path.parent(), path.file_stem()) {
            let shortcuts_path = parent.join(format!("{}_shortcuts.json", stem.to_string_lossy()));
            if shortcuts_path.exists() {
                let file = File::open(&shortcuts_path)?;
                file.lock_shared()?;
                settings.shortcuts = serde_json::from_reader(&file)?;
            }
        }

        settings.migrate()?;
        settings.sanitize()?;
        Ok(settings)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();

        {
            // Create a copy without shortcuts for main settings file
            let mut settings_copy = serde_json::to_value(self)?;
            settings_copy.as_object_mut().unwrap().remove("shortcuts");

            let mut file = File::create(path)?;
            file.lock()?;
            serde_json::to_writer_pretty(&file, &settings_copy)?;
            file.flush()?;
        }

        // Save shortcuts to sibling file
        if let (Some(parent), Some(stem)) = (path.parent(), path.file_stem()) {
            let shortcuts_path = parent.join(format!("{}_shortcuts.json", stem.to_string_lossy()));
            let mut shortcuts_file = File::create(&shortcuts_path)?;
            shortcuts_file.lock()?;
            serde_json::to_writer_pretty(&shortcuts_file, &self.shortcuts)?;
            shortcuts_file.flush()?;
        }

        Ok(())
    }

    /// This indicates if the widget is enabled on general, doesn't take in care multi-instances
    pub fn is_widget_enabled(&self, widget_id: &WidgetId) -> bool {
        self.by_widget.is_enabled(widget_id)
    }

    pub fn set_widget_enabled(&mut self, widget_id: &WidgetId, enabled: bool) {
        self.by_widget.set_enabled(widget_id, enabled);
    }

    pub fn is_widget_enabled_on_monitor(
        &self,
        widget_id: &WidgetId,
        monitor_id: &MonitorId,
    ) -> bool {
        if !self.is_widget_enabled(widget_id) {
            return false;
        }
        // default to true as new connected monitors should be enabled
        self.monitors_v3
            .get(monitor_id)
            .is_none_or(|monitor_config| monitor_config.by_widget.is_widget_enabled(widget_id))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct PerformanceModeSettings {
    pub default: PerformanceMode,
    pub on_battery: PerformanceMode,
    pub on_energy_saver: PerformanceMode,
}

impl Default for PerformanceModeSettings {
    fn default() -> Self {
        Self {
            default: PerformanceMode::Disabled,
            on_battery: PerformanceMode::Minimal,
            on_energy_saver: PerformanceMode::Extreme,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum PerformanceMode {
    /// Does nothing, all animations are enabled.
    Disabled,
    /// Disables windows animations and other heavy effects.
    Minimal,
    /// Disables all the animations.
    Extreme,
}
