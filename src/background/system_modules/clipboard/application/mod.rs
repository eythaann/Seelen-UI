#[cfg(windows)]
mod windows;

use std::{
    sync::{
        LazyLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use seelen_core::system_state::{ClipboardData, ClipboardEntry};
use slu_utils::{Debounce, debounce};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    utils::lock_free::SyncHashMap,
};

use super::domain::{
    CapturedContent, ClipboardEvent, ClipboardStore, ClipboardSysEvent, ClipboardSystem,
};

#[cfg(windows)]
type Platform = windows::WindowsClipboard;

/// Maximum number of entries kept in our own clipboard history.
const MAX_ENTRIES: usize = 200;
/// A single entry larger than this (approximate, in bytes) is dropped instead
/// of being stored, to avoid unbounded memory/disk usage from huge copies.
const MAX_ENTRY_BYTES: usize = 25 * 1024 * 1024;
/// Total approximate size of all stored entries. Oldest entries are evicted
/// once this is exceeded.
const MAX_TOTAL_BYTES: usize = 200 * 1024 * 1024;
/// How long to wait after the last change before persisting to disk.
const PERSIST_DEBOUNCE: Duration = Duration::from_secs(2);

pub struct ClipboardManager {
    platform: Platform,
    /// Our own clipboard history, keyed by our own generated id.
    entries: SyncHashMap<String, ClipboardEntry>,
    /// Ids of entries pinned by the user, kept separate from `entries` so
    /// pin state survives entry eviction/replacement without extra bookkeeping.
    pinned_ids: SyncHashMap<String, ()>,
    /// Whether the OS *native* clipboard history is enabled. Informational only, it doesn't
    /// affect our own capture.
    history_enabled: AtomicBool,
}

event_manager!(ClipboardManager, ClipboardEvent);

impl ClipboardManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<ClipboardManager> = LazyLock::new(|| {
            let mut m = ClipboardManager::create();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
            entries: SyncHashMap::new(),
            pinned_ids: SyncHashMap::new(),
            history_enabled: AtomicBool::new(false),
        }
    }

    fn init(&mut self) -> Result<()> {
        let store = self.platform.load_store();
        log::debug!(
            "[clipboard] init: loaded {} entries ({} pinned) from disk",
            store.items.len(),
            store.pinned_ids.len()
        );
        for entry in store.items {
            self.entries.upsert(entry.id.clone(), entry);
        }
        for id in store.pinned_ids {
            self.pinned_ids.upsert(id, ());
        }

        self.platform.init(Self::on_platform_event)?;
        self.history_enabled
            .store(self.platform.is_history_enabled(), Ordering::SeqCst);
        Ok(())
    }

    fn on_platform_event(event: ClipboardSysEvent) {
        let manager = Self::instance();
        match event {
            ClipboardSysEvent::ContentCaptured { capture } => manager.add_entry(*capture),
            ClipboardSysEvent::HistoryEnabledChanged { enabled } => {
                manager.history_enabled.store(enabled, Ordering::SeqCst);
                Self::send(ClipboardEvent::EnabledChanged);
            }
        }
    }

    pub fn data(&self) -> ClipboardData {
        let mut history = self.entries.values();
        history.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
        ClipboardData {
            history,
            is_history_enabled: self.history_enabled.load(Ordering::SeqCst),
            pinned_ids: self.pinned_ids.keys(),
        }
    }

    /// Notifies the change and persists it (debounced).
    fn entries_changed(&self) {
        Self::send(ClipboardEvent::EntriesChanged);
        Self::request_persist();
    }

    /// Debounces disk writes: repeated calls within [`PERSIST_DEBOUNCE`]
    /// of each other collapse into a single write of the current entries.
    fn request_persist() {
        static DEBOUNCER: LazyLock<Debounce<()>> = LazyLock::new(|| {
            debounce(
                |_| {
                    let manager = ClipboardManager::instance();
                    let store = ClipboardStore {
                        items: manager.entries.values(),
                        pinned_ids: manager.pinned_ids.keys(),
                    };
                    log::debug!(
                        "[clipboard] persisting {} entries to disk",
                        store.items.len()
                    );
                    match manager.platform.save_store(&store) {
                        Ok(()) => {
                            log::debug!("[clipboard] persisted clipboard history successfully")
                        }
                        Err(e) => {
                            log::error!("[clipboard] failed to persist clipboard history: {e}")
                        }
                    }
                },
                PERSIST_DEBOUNCE,
            )
        });
        DEBOUNCER.call(());
    }

    fn add_entry(&self, capture: CapturedContent) {
        let size = capture.content.approx_size();
        if size == 0 {
            log::debug!(
                "[clipboard] skipping empty entry (app={:?})",
                capture.source_app_name
            );
            return;
        }
        if size > MAX_ENTRY_BYTES {
            log::debug!(
                "[clipboard] skipping entry larger than the configured limit ({size} bytes > {MAX_ENTRY_BYTES})"
            );
            return;
        }

        let entry = ClipboardEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: now_ms(),
            source_app_name: capture.source_app_name,
            source_app_umid: capture.source_app_umid,
            source_app_path: capture.source_app_path,
            content: capture.content,
        };
        let fingerprint = entry.content.fingerprint();

        // Look up and (depending on the outcome) update or insert under a
        // single lock, so that two clipboard changes handled concurrently
        // can't both see "no duplicate" and insert the same content twice.
        let is_new = self.entries.with_lock(|map| {
            if let Some((id, existing)) = map
                .iter_mut()
                .find(|(_, e)| e.content.fingerprint() == fingerprint)
            {
                log::debug!(
                    "[clipboard] duplicate content, bumping timestamp of existing entry {id}"
                );
                existing.timestamp = entry.timestamp;
                false
            } else {
                log::debug!(
                    "[clipboard] inserting new entry {} (size={size} bytes, app={:?})",
                    entry.id,
                    entry.source_app_name
                );
                map.insert(entry.id.clone(), entry);
                true
            }
        });

        if is_new {
            self.evict_if_needed();
        }
        self.entries_changed();
        log::debug!("[clipboard] entries count now = {}", self.entries.len());
    }

    /// Evicts the oldest entries until both the entry-count and total-size
    /// limits are satisfied.
    fn evict_if_needed(&self) {
        self.entries.with_lock(|map| {
            loop {
                let over_count = map.len() > MAX_ENTRIES;
                let total_size: usize = map.values().map(|e| e.content.approx_size()).sum();
                let over_size = total_size > MAX_TOTAL_BYTES;

                if !over_count && !over_size {
                    break;
                }

                let Some(oldest_id) = map
                    .values()
                    .min_by_key(|e| e.timestamp)
                    .map(|e| e.id.clone())
                else {
                    break;
                };

                log::debug!(
                    "[clipboard] evicting oldest entry {oldest_id} (over_count={over_count}, over_size={over_size})"
                );
                map.remove(&oldest_id);
            }
        });
    }

    /// Deletes the history entry with the given id from our own store.
    pub fn delete_entry(&self, id: &str) {
        self.entries.remove(id);
        self.pinned_ids.remove(id);
        log::debug!("[clipboard] deleted entry {id}");
        self.entries_changed();
    }

    /// Removes all non-pinned entries from our own clipboard history store.
    pub fn clear_history(&self) {
        self.entries
            .retain(|(id, _)| self.pinned_ids.contains_key(id));
        log::debug!("[clipboard] cleared all non-pinned entries");
        self.entries_changed();
    }

    /// Marks the given entry as pinned, exempting it from [`Self::clear_history`].
    pub fn pin_entry(&self, id: &str) -> Result<()> {
        if !self.entries.contains_key(id) {
            return Err("Clipboard entry not found".into());
        }
        self.pinned_ids.upsert(id.to_owned(), ());
        log::debug!("[clipboard] pinned entry {id}");
        self.entries_changed();
        Ok(())
    }

    /// Removes the pinned status from the given entry.
    pub fn unpin_entry(&self, id: &str) {
        self.pinned_ids.remove(id);
        log::debug!("[clipboard] unpinned entry {id}");
        self.entries_changed();
    }

    /// Sets the given history entry as the current clipboard content.
    ///
    /// When `plain` is `true`, only the plain text is restored, so a
    /// subsequent paste always lands as plain text regardless of what rich
    /// formats (HTML, RTF, bitmap, ...) the entry also has stored.
    pub fn set_clipboard_content(&self, id: &str, plain: bool) -> Result<()> {
        let content = self
            .entries
            .get(id, |e| e.content.clone())
            .ok_or("Clipboard entry not found")?;

        log::debug!("[clipboard] setting clipboard content from entry {id} (plain={plain})");
        self.platform.set_content(&content, plain)?;

        if self.entries.get(id, |e| e.timestamp = now_ms()).is_some() {
            self.entries_changed();
        }
        Ok(())
    }

    /// Sets the entry as clipboard content and pastes it on the focused window.
    pub fn paste(&self, id: &str, plain: bool) -> Result<()> {
        self.set_clipboard_content(id, plain)?;
        self.platform.paste().log_error();
        Ok(())
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}
