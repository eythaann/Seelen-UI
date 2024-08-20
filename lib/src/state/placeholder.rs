use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn default_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

macro_rules! common_item {
    (
        $(
            $(#[$scope:meta])*
            struct $name:ident {
                $($rest:tt)*
            }
        )*
    ) => {
        $(
            #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
            #[serde(rename_all = "camelCase")]
            pub struct $name {
                /// Id to identify the item, should be unique.
                #[serde(default = "default_id")]
                id: String,
                /// Content to display in the item.
                ///
                /// Should follow the [mathjs expression syntax](https://mathjs.org/docs/expressions/syntax.html).
                ///
                $(#[$scope])*
                #[serde(default)]
                template: String,
                /// Content to display in tooltip of the item.
                ///
                /// Should follow the [mathjs expression syntax](https://mathjs.org/docs/expressions/syntax.html).
                ///
                $(#[$scope])*
                tooltip: Option<String>,
                /// Badge will be displayed over the item, useful as notifications.
                ///
                /// Should follow the [mathjs expression syntax](https://mathjs.org/docs/expressions/syntax.html).
                ///
                $(#[$scope])*
                badge: Option<String>,
                /// Deprecated use `onClickV2` instead.
                on_click: Option<String>,
                /// This code will be parsed and executed when the item is clicked.
                ///
                /// Should follow the [mathjs expression syntax](https://mathjs.org/docs/expressions/syntax.html).
                ///
                $(#[$scope])*
                on_click_v2: Option<String>,
                /// Styles to be added to the item. This follow the same interface of React's `style` prop.
                #[serde(default)]
                style: HashMap<String, Option<StyleValue>>,
                $($rest)*
            }
        )*
    };
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum StyleValue {
    String(String),
    PosInt(u64),
    /// Always less than zero.
    NegInt(i64),
    /// Always finite.
    Float(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum DateUpdateInterval {
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum WorkspaceToolbarItemMode {
    Dotted,
    Named,
    Numbered,
}

common_item! {
    /// ## Base Item Scope
    /// Have all icons defined on [React Icons](https://react-icons.github.io/react-icons) as properties of the object.
    /// ```js
    /// const icon: object;
    /// ```
    /// Haves all environment variables defined on the system as properties of the object.
    /// ```js
    /// const env: object;
    /// ```
    /// Functions to add images to the item.
    /// ```js
    /// function imgFromUrl (url: string, size: number = 16): string
    /// function imgFromPath (path: string, size: number = 16): string
    /// function imgFromExe (exe_path: string, size: number = 16): string
    /// ```
    struct TextToolbarItem {}

    /// ## Generic Item Scope
    /// ```ts
    /// // the current focused window
    /// const window: {
    ///     name: string;
    ///     title: string;
    ///     exe: string | null;
    /// };
    /// ```
    struct GenericToolbarItem {}

    /// ## Date Item Scope
    /// ```ts
    /// const date: string; // the formatted date
    /// ```
    struct DateToolbarItem {
        /// Time unit to refresh the showing date
        #[serde(default = "DateToolbarItem::default_interval")]
        each: DateUpdateInterval,
        /// Format of the date, see [moment.js displaying format](https://momentjs.com/docs/#/displaying/format/)
        #[serde(default = "DateToolbarItem::default_format")]
        format: String,
    }

    /// ## Power Item Scope
    /// ```ts
    /// interface PowerStatus {
    ///     acLineStatus: number;
    ///     batteryFlag: number;
    ///     batteryLifePercent: number;
    ///     systemStatusFlag: number;
    ///     batteryLifeTime: number;
    ///     batteryFullLifeTime: number;
    /// }
    ///
    /// interface Battery {
    ///     // Static info
    ///     vendor: string | null;
    ///     model: string | null;
    ///     serialNumber: string | null;
    ///     technology: string;
    ///
    ///     // Common information
    ///     state: string;
    ///     capacity: number;
    ///     temperature: number | null;
    ///     percentage: number;
    ///     cycleCount: number | null;
    ///     smartCharging: boolean;
    ///
    ///     // Energy stats
    ///     energy: number;
    ///     energyFull: number;
    ///     energyFullDesign: number;
    ///     energyRate: number;
    ///     voltage: number;
    ///
    ///     // Charge stats
    ///     timeToFull: number | null;
    ///     timeToEmpty: number | null;
    /// }
    ///
    /// const power: PowerStatus;
    /// const batteries: Battery[];
    /// const battery: Battery | null;
    /// ```
    struct PowerToolbarItem {}

    /// ## Network Item Scope
    /// ```ts
    /// interface NetworkInterface {
    ///     name: string;
    ///     description: string;
    ///     status: 'up' | 'down';
    ///     dnsSuffix: string;
    ///     type: string;
    ///     gateway: string | null;
    ///     mac: string;
    ///     ipv4: string | null;
    ///     ipv6: string | null;
    /// }
    /// const online: boolean;
    /// const interfaces: NetworkInterface[];
    /// const usingInterface: NetworkInterface | null;
    /// ```
    struct NetworkToolbarItem {
        /// Show Wi-fi selector popup on click]
        #[serde(default)]
        with_wlan_selector: bool,
    }

    /// ## Media Item Scope
    /// ```ts
    /// const volume: number; // output master volume from 0 to 1
    /// const isMuted: boolean; // output master volume is muted
    /// const inputVolume: number; // input master volume from 0 to 1
    /// const inputIsMuted: boolean; // input master volume is muted
    ///
    /// interface MediaSession {
    ///     id: string;
    ///     title: string;
    ///     author: string;
    ///     thumbnail: string | null; // path to temporal media session image
    ///     playing: boolean;
    ///     default: boolean;
    /// }
    ///
    /// const mediaSession: MediaSession | null;
    /// ```
    struct MediaToolbarItem {
        /// Show media controls popup on click
        #[serde(default)]
        with_media_controls: bool,
    }

    /// ## Notifications Item Scope
    /// ```ts
    /// const count: number;
    /// ```
    struct NotificationsToolbarItem {}

    /// ## Workspace Item Scope
    /// this module does no expand the scope of the item
    struct TrayToolbarItem {}

    /// ## Device Item Scope
    /// this module does no expand the scope of the item
    struct DeviceToolbarItem {}

    /// ## Settings Item Scope
    /// this module does no expand the scope of the item
    struct SettingsToolbarItem {}

    /// ## Workspace Item Scope
    /// this module does no expand the scope of the item
    struct WorkspaceToolbarItem {
        #[serde(default = "WorkspaceToolbarItem::default_mode")]
        mode: WorkspaceToolbarItemMode,
    }
}

impl DateToolbarItem {
    fn default_interval() -> DateUpdateInterval {
        DateUpdateInterval::Minute
    }

    fn default_format() -> String {
        "MMM Do, HH:mm".to_string()
    }
}

impl WorkspaceToolbarItem {
    fn default_mode() -> WorkspaceToolbarItemMode {
        WorkspaceToolbarItemMode::Dotted
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ToolbarItem {
    Text(TextToolbarItem),
    Generic(GenericToolbarItem),
    Date(DateToolbarItem),
    Power(PowerToolbarItem),
    Network(NetworkToolbarItem),
    Media(MediaToolbarItem),
    Notifications(NotificationsToolbarItem),
    Tray(TrayToolbarItem),
    Device(DeviceToolbarItem),
    Settings(SettingsToolbarItem),
    Workspaces(WorkspaceToolbarItem),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct PlaceholderInfo {
    /// Display name of the placeholder
    pub display_name: String,
    /// Author of the placeholder
    pub author: String,
    /// Description of the placeholder
    pub description: String,
    /// Filename of the placeholder, is overridden by the program on load.
    pub filename: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct Placeholder {
    /// Metadata about the placeholder
    pub info: PlaceholderInfo,
    /// Items to be displayed in the toolbar
    pub left: Vec<ToolbarItem>,
    /// Items to be displayed in the toolbar
    pub center: Vec<ToolbarItem>,
    /// Items to be displayed in the toolbar
    pub right: Vec<ToolbarItem>,
}
