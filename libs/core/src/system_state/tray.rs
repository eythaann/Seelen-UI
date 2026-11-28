use std::path::PathBuf;

use super::WindowIdentity;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
pub struct SysTrayIcon {
    /// Identifier of the icon during the session, as the shell identifies it: its guid, or
    /// its (window handle + uid). Will not change for the lifetime of the icon.
    pub session_id: String,

    /// Persistent identifier of the icon: its guid, or the executable that registered it plus
    /// its uid. Unlike the session id it survives sessions, and it is shared by every instance
    /// of a multi-instance app.
    ///
    /// Identifies the icon settings stored by Seelen (pinned state and order), `None` when the
    /// executable of the icon is unknown, in that case the icon can't be pinned nor sorted.
    #[serde(default)]
    pub key: Option<String>,

    /// Path of the executable that registered the icon.
    pub executable_path: PathBuf,

    /// Application-defined identifier for the icon, used in combination
    /// with the window handle.
    ///
    /// The uid only has to be unique for the window handle. Multiple
    /// icons (across different window handles) can have the same uid.
    pub uid: Option<u32>,

    /// Handle to the window that contains the icon. Used in combination
    /// with a uid.
    ///
    /// Note that multiple icons can have the same window handle.
    pub window_handle: Option<WindowIdentity>,

    /// GUID for the icon.
    ///
    /// Used as an alternate way to identify the icon (versus its window
    /// handle and uid).
    pub guid: Option<uuid::Uuid>,

    /// Tooltip to show for the icon on hover.
    pub tooltip: String,

    /// Handle to the icon bitmap.
    pub icon_handle: Option<isize>,

    /// Path to the icon image file.
    pub icon_path: Option<PathBuf>,

    /// Hash of the icon image.
    ///
    /// Used to determine if the icon image has changed without having to
    /// compare the entire image.
    pub icon_image_hash: Option<String>,

    /// Whether the icon image is a single color glyph (all visible pixels share the same
    /// color, the shape lives in the alpha channel). Shell icons like "Safely Remove Hardware"
    /// are drawn this way for the taskbar theme, so the UI should use them as alpha mask.
    pub is_glyph: bool,

    /// Application-defined message identifier.
    ///
    /// Used to send messages to the window that contains the icon.
    pub callback_message: Option<u32>,

    /// Version of the icon.
    pub version: Option<u32>,

    /// Whether the icon is visible in the system tray.
    ///
    /// This is determined by the `NIS_HIDDEN` flag in the icon's state.
    pub is_visible: bool,

    /// Whether the icon is shown directly on the toolbar, otherwise it is only shown in
    /// the tray popup. Stored by Seelen per `key`, always `false` for icons without key.
    pub is_promoted: bool,
}

/// Actions that can be performed on a `SystrayIcon`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(repr(enum = name)))]
pub enum SystrayIconAction {
    HoverEnter,
    HoverLeave,
    HoverMove,
    LeftClick,
    RightClick,
    MiddleClick,
    LeftDoubleClick,
}
