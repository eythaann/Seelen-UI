use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
pub struct SysTrayIcon {
    /// Persistent identifier assigned by Windows to the icon: the name of its subkey under
    /// `HKCU\Control Panel\NotifyIconSettings`, bound to the executable that registered the
    /// icon plus its uid or guid. Unlike the (window handle + uid) pair it survives sessions.
    ///
    /// It is a u64 on Windows, exposed as string because it exceeds the JS safe integer range.
    pub registry_key: String,

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
    pub window_handle: Option<isize>,

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

    /// Whether the icon is shown directly on the taskbar, otherwise it is only
    /// shown in the tray overflow. Stored by Windows as `IsPromoted` on the icon's
    /// registry entry.
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
