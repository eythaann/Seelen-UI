use std::path::PathBuf;

/// Identifier for a systray icon.
///
/// A systray icon is either identified by a (window handle + uid) or
/// its guid. Since a systray icon can be updated to also include a
/// guid or window handle/uid later on, a stable ID is useful for
/// consistently identifying an icon.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize, TS)]
pub enum SysTrayIconId {
    HandleUid(isize, u32),
    Guid(uuid::Uuid),
}

impl std::fmt::Display for SysTrayIconId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SysTrayIconId::HandleUid(handle, uid) => write!(f, "{:x}_{}", handle, uid),
            SysTrayIconId::Guid(guid) => write!(f, "{}", guid),
        }
    }
}

impl std::str::FromStr for SysTrayIconId {
    type Err = crate::error::SeelenLibError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try parsing as handle and uid (format: "handle:uid").
        if let Some((handle_str, uid_str)) = s.split_once(':') {
            return Ok(SysTrayIconId::HandleUid(
                handle_str.parse().map_err(|_| "Invalid icon id")?,
                uid_str.parse().map_err(|_| "Invalid icon id")?,
            ));
        }

        // Try parsing as a guid.
        if let Ok(guid) = uuid::Uuid::parse_str(s) {
            return Ok(SysTrayIconId::Guid(guid));
        }

        Err("Invalid icon id".into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SysTrayIcon {
    /// Identifier for the icon. Will not change for the lifetime of the
    /// icon.
    ///
    /// The Windows shell uses either a (window handle + uid) or its guid
    /// to identify which icon to operate on.
    ///
    /// Read more: https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ns-shellapi-notifyicondataw
    pub stable_id: SysTrayIconId,

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
}

/// Actions that can be performed on a `SystrayIcon`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, repr(enum = name))]
pub enum SystrayIconAction {
    HoverEnter,
    HoverLeave,
    HoverMove,
    LeftClick,
    RightClick,
    MiddleClick,
    LeftDoubleClick,
}
