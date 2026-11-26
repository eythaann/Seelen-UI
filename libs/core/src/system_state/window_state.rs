/// Show state requested for a window of another app.
///
/// Serialized as a number (see [`WindowState::code`]): the IPC between the app and the service sent
/// these values as raw `SW_*` Win32 show commands before, so the codes are kept for compatibility
/// between versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "i32", try_from = "i32")]
pub enum WindowState {
    /// Activates and shows the window.
    Shown,
    /// Shows the window without activating it.
    ShownNoActivate,
    Hidden,
    /// Activates and shows the window in its normal (not minimized nor maximized) state.
    Normal,
    Minimized,
    /// Minimizes even if the thread owning the window is not responding.
    ForceMinimized,
    Maximized,
    /// Back to the previous size and position (unminimize/unmaximize).
    Restored,
}

impl WindowState {
    const ALL: [Self; 8] = [
        Self::Shown,
        Self::ShownNoActivate,
        Self::Hidden,
        Self::Normal,
        Self::Minimized,
        Self::ForceMinimized,
        Self::Maximized,
        Self::Restored,
    ];

    /// Serialized value, frozen: equal to the matching Win32 `SW_*` command.
    pub const fn code(self) -> i32 {
        match self {
            Self::Hidden => 0,
            Self::Normal => 1,
            Self::Maximized => 3,
            Self::ShownNoActivate => 4,
            Self::Shown => 5,
            Self::Minimized => 6,
            Self::Restored => 9,
            Self::ForceMinimized => 11,
        }
    }
}

impl From<WindowState> for i32 {
    fn from(state: WindowState) -> Self {
        state.code()
    }
}

impl TryFrom<i32> for WindowState {
    type Error = String;

    fn try_from(code: i32) -> Result<Self, Self::Error> {
        Self::ALL
            .into_iter()
            .find(|state| state.code() == code)
            .ok_or_else(|| format!("Unknown window state code: {code}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_round_trip() {
        for state in WindowState::ALL {
            assert_eq!(WindowState::try_from(state.code()), Ok(state));
        }
        assert!(WindowState::try_from(2).is_err());
    }

    #[test]
    fn serializes_as_legacy_show_command() {
        assert_eq!(serde_json::to_string(&WindowState::Restored).unwrap(), "9");
        assert_eq!(
            serde_json::from_str::<WindowState>("11").unwrap(),
            WindowState::ForceMinimized
        );
    }
}
