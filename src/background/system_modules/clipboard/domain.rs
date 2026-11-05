use seelen_core::system_state::{ClipboardEntry, ClipboardEntryContent};
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// OS contract of the clipboard module.
pub trait ClipboardSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Starts listening the clipboard, reported through `on_event`. Changes made through
    /// [`Self::set_content`] are not reported.
    /// Called once; the subscriptions live as long as `self`.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(ClipboardSysEvent) + Send + Sync + 'static;

    /// Whether the OS own clipboard history is enabled (informational only).
    fn is_history_enabled(&self) -> bool;

    /// Places the content on the clipboard. `plain` only places the plain text.
    fn set_content(&self, content: &ClipboardEntryContent, plain: bool) -> Result<()>;

    /// Sends the paste shortcut to the focused window.
    fn paste(&self) -> Result<()>;

    /// Reads the persisted history. Empty if there is none or it can't be read.
    fn load_store(&self) -> ClipboardStore;

    /// Persists the history, readable only by the current user.
    fn save_store(&self, store: &ClipboardStore) -> Result<()>;
}

#[derive(Debug, Clone)]
pub enum ClipboardSysEvent {
    /// Another app changed the clipboard.
    ContentCaptured {
        capture: Box<CapturedContent>,
    },
    HistoryEnabledChanged {
        enabled: bool,
    },
}

#[derive(Debug, Clone)]
pub enum ClipboardEvent {
    EntriesChanged,
    EnabledChanged,
}

/// Clipboard content and the app it came from.
#[derive(Debug, Clone)]
pub struct CapturedContent {
    pub content: ClipboardEntryContent,
    pub source_app_name: Option<String>,
    pub source_app_umid: Option<String>,
    pub source_app_path: Option<String>,
}

/// On-disk shape of our clipboard history store. Wrapped in a struct (instead
/// of persisting a bare `Vec<ClipboardEntry>`) so new top-level fields (like
/// `pinned_ids`) can be added later without changing the storage format.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ClipboardStore {
    pub items: Vec<ClipboardEntry>,
    /// Ids of entries pinned by the user. Pinned entries are kept when the
    /// history is cleared.
    pub pinned_ids: Vec<String>,
}
