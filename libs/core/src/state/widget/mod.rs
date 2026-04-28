pub mod context_menu;
pub mod declaration;

use std::collections::HashMap;

use declaration::WidgetSettingsDeclarationList;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    error::Result,
    resource::{ResourceKind, ResourceMetadata, ResourceText, SluResource, WidgetId},
    state::Plugin,
    system_state::MonitorId,
    utils::TsUnknown,
    Point,
};

/// Declares a shortcut that belongs to this widget.
/// The user can override the keys via the widget's `$shortcuts` setting entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct WidgetShortcutDeclaration {
    /// Stable identifier for this shortcut (e.g. `"wm-focus-up"`).
    /// Used as the key for user overrides in `$shortcuts`.
    pub id: String,
    /// CLI command to execute when the shortcut is triggered (e.g. `["wm", "focus", "up"]`).
    pub command: Vec<String>,
    /// Human-readable label shown in the shortcuts settings UI.
    pub label: ResourceText,
    /// Optional longer description shown in the settings UI.
    pub description: Option<ResourceText>,
    /// Default key combination (can be overridden by the user unless `readonly` is true).
    pub default_keys: Vec<String>,
    /// If true, the user cannot change the key combination for this shortcut.
    pub readonly: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct Widget {
    /// Resource id ex: `@seelen/weg`
    pub id: WidgetId,
    /// Optional icon to be used on settings. This have to be a valid react icon name.\
    /// You can find all icons here: https://react-icons.github.io/react-icons/.
    pub icon: Option<String>,
    /// Widget metadata, as texts, tags, images, etc.
    pub metadata: ResourceMetadata,

    /// Widget settings declaration, this is esentially a struct to be used by an
    /// builder to create the widget settings UI on the Settings window.
    pub settings: WidgetSettingsDeclarationList,
    /// Widget configuration preset
    pub preset: WidgetPreset,
    /// If true, the widget webview won't be created until it is requested via trigger action.
    pub lazy: bool,
    /// How many instances are allowed of this widget.
    pub instances: WidgetInstanceMode,
    /// If true, the widget will not be shown on the Settings Navigation as a Tab, but it will
    /// still be available on the widgets full list.
    pub hidden: bool,
    /// If true, the memory leak of webview2 (https://github.com/tauri-apps/tauri/issues/4026)
    /// workaround, will be no applied for instances of this widget.
    pub no_memory_leak_workaround: bool,
    /// will open dev tools inmediately when webview is created
    pub debug: bool,

    /// Way to load the widget
    pub loader: WidgetLoader,
    /// Optional widget js code
    pub js: Option<String>,
    /// Optional widget css
    pub css: Option<String>,
    /// Optional widget html
    pub html: Option<String>,

    /// Optional list of plugins to be installed side the widget.
    /// Use this if your widget needs interaction with other widgets like
    /// adding a context menu item that shows this widget.
    pub plugins: Vec<Plugin>,
    /// Shortcut declarations for this widget.
    /// These define the default shortcuts that the user can override via `$shortcuts` in
    /// the widget's settings entry.
    pub shortcuts: Vec<WidgetShortcutDeclaration>,
}

impl SluResource for Widget {
    const KIND: ResourceKind = ResourceKind::Widget;

    fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut ResourceMetadata {
        &mut self.metadata
    }

    fn validate(&self) -> Result<()> {
        if self.settings.there_are_duplicates() {
            return Err("Widget settings declaration have duplicated keys".into());
        }
        for plugin in &self.plugins {
            plugin.validate()?
        }
        Ok(())
    }

    fn sanitize(&mut self) {
        for plugin in &mut self.plugins {
            plugin.sanitize()
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum WidgetInstanceMode {
    /// Default behavior, only one instance of this widget is allowed.
    /// This is useful for widgets intended to work as custom config window.
    #[default]
    Single,
    /// The widget is allowed to have multiple instances.\
    /// This allow to the user manually create more instances of this same widget.
    Multiple,
    /// Seelen UI will create an instance of this widget per each monitor connected.\
    /// This can be configured by the user using per monitor settings.\
    ReplicaByMonitor,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum WidgetLoader {
    /// Used for old internal widgets, similar to `Internal`.
    Legacy,
    /// Used for internal bundled widgets, this will load the code from internal resources.
    InternalReact,
    /// Used for internal bundled widgets, this will load the code from internal resources.
    Internal,
    /// Used for third party widgets, this will load the code from the `js`, `css`, and `html` fields
    #[default]
    ThirdParty,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum WidgetPreset {
    /// No special behavior, all should be manually configured
    #[default]
    None,
    /// Always on bottom, no title bar, etc. Resizable by default.
    Desktop,
    /// Always on top, no title bar, etc.
    Overlay,
    /// Same as overlay, but will be automatically closed on unfocus;
    /// Also this type of widgets can be manually open/closed/show/hide by other widgets or plugins.
    /// On show the widget will be at the specified position, could be custom one, or will take the mouse cursor position.
    ///
    /// If a widget is of this type, the enabled property won't determine the visibility of the widget,
    /// as this widget is only shown when explicitly requested.
    ///
    /// Widget instances mode will be ignored for this type of widgets, As popups should be always single instance.
    Popup,
}

/// Arguments that could be passed on the trigger widget function, widgets decides if use it or not.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export, optional_fields = nullable))]
pub struct WidgetTriggerPayload {
    pub id: WidgetId,
    pub monitor_id: Option<MonitorId>,
    pub instance_id: Option<uuid::Uuid>,
    /// Desired position to show the widget
    pub desired_position: Option<Point>,
    /// This will be used to align the widget at the desired position
    /// - start will set the widget at the left of point,
    /// - center will set the widget at the center of point,
    /// - end will set the widget at the right of point
    pub align_x: Option<Alignment>,
    /// This will be used to align the widget at the desired position
    /// - start will set the widget at the top of point,
    /// - center will set the widget at the center of point,
    /// - end will set the widget at the bottom of point
    pub align_y: Option<Alignment>,
    /// Custom arguments to be used by the widget recieving the trigger.
    /// this can be anything, and depends on the widget to evaluate them.
    pub custom_args: Option<HashMap<String, TsUnknown>>,
}

impl WidgetTriggerPayload {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            monitor_id: None,
            instance_id: None,
            desired_position: None,
            align_x: None,
            align_y: None,
            custom_args: None,
        }
    }

    pub fn add_custom_arg(&mut self, key: impl AsRef<str>, value: impl Into<TsUnknown>) {
        if self.custom_args.is_none() {
            self.custom_args = Some(HashMap::new());
        }
        self.custom_args
            .as_mut()
            .unwrap()
            .insert(key.as_ref().to_string(), value.into());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(repr(enum = name))]
pub enum Alignment {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[cfg_attr(feature = "gen-binds", ts(export, repr(enum = name)))]
pub enum WidgetStatus {
    /// Widget has been registered, but not yet loaded
    Pending,
    /// Webview window is being created
    Creating,
    /// Widget javascript, html and css is being loaded
    Mounting,
    /// Widget loaded and is ready
    Ready,
    /// Webview window failed to be created.
    CrashedOnCreation,
    /// Webview window is being restarted
    Restarting,
}
