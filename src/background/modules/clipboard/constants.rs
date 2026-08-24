/// Maximum number of entries kept in our own clipboard history.
pub const CLIPBOARD_MAX_ENTRIES: usize = 200;

/// A single entry larger than this (approximate, in bytes) is dropped instead
/// of being stored, to avoid unbounded memory/disk usage from huge copies.
pub const CLIPBOARD_MAX_ENTRY_BYTES: usize = 25 * 1024 * 1024;

/// Total approximate size of all stored entries. Oldest entries are evicted
/// once this is exceeded.
pub const CLIPBOARD_MAX_TOTAL_BYTES: usize = 200 * 1024 * 1024;

/// How long to wait after the last change before persisting to disk.
pub const CLIPBOARD_PERSIST_DEBOUNCE: std::time::Duration = std::time::Duration::from_secs(2);
