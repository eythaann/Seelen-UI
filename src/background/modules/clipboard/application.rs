use std::sync::{
    LazyLock, OnceLock,
    atomic::{AtomicBool, AtomicU32, Ordering},
};

use seelen_core::system_state::{ClipboardData, ClipboardEntry, ClipboardEntryContent};
use slu_utils::{Debounce, debounce};
use windows::{
    ApplicationModel::DataTransfer::{
        Clipboard, DataPackage, DataPackageOperation, DataPackageView, StandardDataFormats,
    },
    Foundation::{EventHandler, Uri},
    Storage::{IStorageItem, StorageFile, StorageFolder},
    Win32::{
        System::DataExchange::GetClipboardOwner,
        UI::WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, MSG, TranslateMessage, WM_CLIPBOARDUPDATE,
        },
    },
    core::HSTRING,
};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    modules::clipboard::{
        constants::{
            CLIPBOARD_MAX_ENTRIES, CLIPBOARD_MAX_ENTRY_BYTES, CLIPBOARD_MAX_TOTAL_BYTES,
            CLIPBOARD_PERSIST_DEBOUNCE,
        },
        persistence::{self, ClipboardStore},
    },
    utils::lock_free::SyncHashMap,
    windows_api::{Com, WindowsApi, event_window::subscribe_to_background_window, window::Window},
};

/// Runs `f` on a fresh STA thread and returns its result.
/// Blocks the calling thread until the spawned thread finishes.
fn sta_call<F, T>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        tx.send(Com::run_with_context(f)).ok();
    });
    rx.recv().map_err(|e| e.to_string())?
}

/// Runs `f` on a fresh STA thread, fire-and-forget: does not block the
/// caller, just logs `context` on failure. Use for work whose result no one
/// is waiting on (e.g. reacting to an incoming event), as opposed to
/// [`sta_call`] which is for work the caller needs the result of.
fn spawn_sta<F>(context: &'static str, f: F)
where
    F: FnOnce() -> Result<()> + Send + 'static,
{
    std::thread::spawn(move || {
        if let Err(e) = Com::run_with_context(f) {
            log::error!("[clipboard] {context} failed: {e}");
        }
    });
}

/// Counts clipboard writes we ourselves originated (via [`ClipboardManager::set_clipboard_content`])
/// that haven't yet been observed via `WM_CLIPBOARDUPDATE`. Incremented right
/// before we call `Clipboard::SetContent`, decremented when the resulting
/// notification arrives.
///
/// This can't be done by checking the clipboard owner window instead, because
/// restoring an entry can be triggered by our own API/CLI without any of our
/// windows being focused (or existing at all) at that moment, so there's no
/// reliable "our window owns the clipboard" signal to check against.
static PENDING_SELF_SETS: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone)]
pub enum ClipboardEvent {
    EntriesChanged,
    EnabledChanged,
}

pub struct ClipboardManager {
    /// Our own clipboard history, keyed by our own generated id.
    pub entries: SyncHashMap<String, ClipboardEntry>,
    /// Ids of entries pinned by the user, kept separate from `entries` so
    /// pin state survives entry eviction/replacement without extra bookkeeping.
    pub pinned_ids: SyncHashMap<String, ()>,
    /// Whether Windows' *native* clipboard history is enabled. Kept only for
    /// debugging/informational purposes — it no longer affects our capture,
    /// since we listen to `WM_CLIPBOARDUPDATE` directly instead of the native
    /// history API.
    pub history_enabled: AtomicBool,

    history_enabled_token: Option<i64>,
}

unsafe impl Send for ClipboardManager {}
unsafe impl Sync for ClipboardManager {}

event_manager!(ClipboardManager, ClipboardEvent);

impl ClipboardManager {
    fn new() -> Self {
        Self {
            entries: SyncHashMap::new(),
            pinned_ids: SyncHashMap::new(),
            history_enabled: AtomicBool::new(false),
            history_enabled_token: None,
        }
    }

    /// Debounces disk writes: repeated calls within [`CLIPBOARD_PERSIST_DEBOUNCE`]
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
                    match persistence::save_store(&store) {
                        Ok(()) => {
                            log::debug!("[clipboard] persisted clipboard history successfully")
                        }
                        Err(e) => {
                            log::error!("[clipboard] failed to persist clipboard history: {e}")
                        }
                    }
                },
                CLIPBOARD_PERSIST_DEBOUNCE,
            )
        });
        DEBOUNCER.call(());
    }

    pub fn instance() -> &'static Self {
        static MANAGER: OnceLock<ClipboardManager> = OnceLock::new();
        if MANAGER.get().is_none() {
            log::debug!("[clipboard] starting ClipboardManager STA thread");

            // Clipboard WinRT APIs require an STA thread. Spawn a dedicated one
            // that initialises COM as STA, registers event listeners, then runs a
            // Windows message pump so that WinRT can dispatch event callbacks.
            let (ready_tx, ready_rx) = std::sync::mpsc::channel::<()>();
            std::thread::spawn(move || {
                let _ = Com::run_with_context(|| {
                    MANAGER.get_or_init(|| {
                        let mut m = ClipboardManager::new();
                        m.init().log_error();
                        m
                    });

                    // Signal that the manager is ready before entering the pump.
                    ready_tx.send(()).ok();

                    // Pump messages so WinRT delivers HistoryEnabledChanged.
                    let mut msg = MSG::default();
                    loop {
                        if unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
                            unsafe {
                                let _ = TranslateMessage(&msg);
                                DispatchMessageW(&msg);
                            }
                        } else {
                            break;
                        }
                    }
                    Ok(())
                });
            });

            ready_rx.recv().ok();
        }
        MANAGER.get().expect("clipboard manager not initialised")
    }

    pub fn get_data(&self) -> ClipboardData {
        let mut history = self.entries.values();
        history.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
        ClipboardData {
            history,
            is_history_enabled: self.history_enabled.load(Ordering::SeqCst),
            pinned_ids: self.pinned_ids.keys(),
        }
    }

    /// Deletes the history entry with the given id from our own store.
    pub fn delete_entry(id: &str) -> Result<()> {
        let manager = Self::instance();
        manager.entries.remove(id);
        manager.pinned_ids.remove(id);
        log::debug!("[clipboard] deleted entry {id}");
        Self::send(ClipboardEvent::EntriesChanged);
        Self::request_persist();
        Ok(())
    }

    /// Removes all non-pinned entries from our own clipboard history store.
    pub fn clear_history() -> Result<()> {
        let manager = Self::instance();
        manager
            .entries
            .retain(|(id, _)| manager.pinned_ids.contains_key(id));
        log::debug!("[clipboard] cleared all non-pinned entries");
        Self::send(ClipboardEvent::EntriesChanged);
        Self::request_persist();
        Ok(())
    }

    /// Marks the given entry as pinned, exempting it from [`Self::clear_history`].
    pub fn pin_entry(id: &str) -> Result<()> {
        let manager = Self::instance();
        if !manager.entries.contains_key(id) {
            return Err("Clipboard entry not found".into());
        }
        manager.pinned_ids.upsert(id.to_owned(), ());
        log::debug!("[clipboard] pinned entry {id}");
        Self::send(ClipboardEvent::EntriesChanged);
        Self::request_persist();
        Ok(())
    }

    /// Removes the pinned status from the given entry.
    pub fn unpin_entry(id: &str) -> Result<()> {
        let manager = Self::instance();
        manager.pinned_ids.remove(id);
        log::debug!("[clipboard] unpinned entry {id}");
        Self::send(ClipboardEvent::EntriesChanged);
        Self::request_persist();
        Ok(())
    }

    /// Sets the given history entry as the current Windows clipboard content,
    /// rebuilding a `DataPackage` from what we have stored for it.
    ///
    /// When `plain` is `true`, only the plain text is restored, so a
    /// subsequent paste always lands as plain text regardless of what rich
    /// formats (HTML, RTF, bitmap, ...) the entry also has stored.
    pub fn set_clipboard_content(id: &str, plain: bool) -> Result<()> {
        let entry = Self::instance()
            .entries
            .get(id, |e| e.clone())
            .ok_or("Clipboard entry not found")?;

        log::debug!("[clipboard] setting clipboard content from entry {id} (plain={plain})");
        let result = sta_call(move || {
            let package = DataPackage::new()?;
            package.SetRequestedOperation(DataPackageOperation::Copy)?;

            let content = &entry.content;
            let mut set_something = false;

            if let Some(text) = &content.text {
                package.SetText(&HSTRING::from(text.as_str()))?;
                set_something = true;
            }

            if !plain {
                if let Some(html) = &content.html {
                    package.SetHtmlFormat(&HSTRING::from(html.as_str()))?;
                    set_something = true;
                }
                if let Some(rtf) = &content.rtf {
                    package.SetRtf(&HSTRING::from(rtf.as_str()))?;
                    set_something = true;
                }
                if let Some(bitmap) = &content.bitmap {
                    let stream_ref = WindowsApi::webp_base64_to_random_access_stream_ref(bitmap)?;
                    package.SetBitmap(&stream_ref)?;
                    set_something = true;
                }
                if let Some(link) = &content.application_link {
                    package.SetApplicationLink(&Uri::CreateUri(&HSTRING::from(link.as_str()))?)?;
                    set_something = true;
                }
                if let Some(link) = &content.web_link {
                    package.SetWebLink(&Uri::CreateUri(&HSTRING::from(link.as_str()))?)?;
                    set_something = true;
                }
            }

            if let Some(paths) = &content.files {
                let mut items = Vec::with_capacity(paths.len());
                for path in paths {
                    let item = Self::storage_item_from_path(path)?;
                    items.push(Some(item));
                }
                let items = windows_collections::IIterable::<IStorageItem>::from(items);
                package.SetStorageItems(&items, false)?;
                set_something = true;
            }

            if !set_something {
                return Err("This entry has no restorable content".into());
            }

            // Mark this write as self-originated *before* handing it to the
            // OS, so the `WM_CLIPBOARDUPDATE` it triggers is guaranteed to
            // see the incremented counter whenever it arrives.
            PENDING_SELF_SETS.fetch_add(1, Ordering::SeqCst);
            let result = Clipboard::SetContent(&package).and_then(|_| Clipboard::Flush());
            if result.is_err() {
                // The write never reached the clipboard, so no matching
                // notification will ever arrive to consume this count.
                PENDING_SELF_SETS.fetch_sub(1, Ordering::SeqCst);
            }
            Ok(result?)
        });

        if result.is_ok() {
            let manager = Self::instance();
            if manager
                .entries
                .get(id, |e| e.timestamp = now_ms())
                .is_some()
            {
                Self::send(ClipboardEvent::EntriesChanged);
                Self::request_persist();
            }
        }

        result
    }

    /// Resolves a filesystem path back into an `IStorageItem`, for restoring
    /// file/folder clipboard entries.
    fn storage_item_from_path(path: &str) -> Result<IStorageItem> {
        use windows_core::Interface;

        let is_dir = std::path::Path::new(path).is_dir();
        let path = HSTRING::from(path);
        if is_dir {
            let folder = StorageFolder::GetFolderFromPathAsync(&path)?.join()?;
            Ok(folder.cast::<IStorageItem>()?)
        } else {
            let file = StorageFile::GetFileFromPathAsync(&path)?.join()?;
            Ok(file.cast::<IStorageItem>()?)
        }
    }

    /// Processes the current WinRT clipboard view into a [`ClipboardEntry`].
    /// `owner_hwnd` is the clipboard owner window (via `GetClipboardOwner`) captured
    /// at the time of the clipboard change, used to resolve the source app's name,
    /// app user model id and executable path. Must be called from an STA-initialised
    /// thread.
    fn process_view(view: &DataPackageView, owner_hwnd: isize) -> Result<ClipboardEntry> {
        let (source_app_name, source_app_umid, source_app_path) = if owner_hwnd != 0 {
            let window = Window::from(owner_hwnd);
            let name = window.app_display_name().ok().filter(|s| !s.is_empty());
            let umid = window.app_user_model_id().map(|umid| umid.to_string());
            let path = window
                .process()
                .program_path()
                .ok()
                .map(|p| p.to_string_lossy().to_string());
            (name, umid, path)
        } else {
            (None, None, None)
        };

        // Fall back to the WinRT-provided application name, which is only ever
        // populated when the source app explicitly set it via `DataPackage`
        // (mostly packaged apps using the WinRT clipboard APIs directly).
        let source_app_name = source_app_name.or_else(|| {
            view.Properties()
                .ok()?
                .ApplicationName()
                .ok()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
        });

        let content = get_content_from_view(view)?;

        Ok(ClipboardEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: now_ms(),
            source_app_name,
            source_app_umid,
            source_app_path,
            content,
        })
    }

    /// Called for every message received by the shared background window.
    /// WinRT's `Clipboard.ContentChanged` event does not reliably fire for
    /// unpackaged Win32 apps like this one, so we instead rely on the classic
    /// `WM_CLIPBOARDUPDATE` notification (see `AddClipboardFormatListener` in
    /// `windows_api::event_window`), which works regardless of package identity.
    fn on_bg_window_proc(msg: u32, _w_param: usize, _l_param: isize) -> Result<()> {
        if msg != WM_CLIPBOARDUPDATE {
            return Ok(());
        }
        log::debug!("[clipboard] WM_CLIPBOARDUPDATE received");

        // Consume one self-set credit, if any, instead of processing this as
        // an externally-originated change. This is what stops restoring an
        // entry (`set_clipboard_content`) from being re-captured as a brand
        // new entry, which for bitmaps also caused a decode/re-encode loop
        // that degraded the image a little more on every restore.
        let was_self_set = PENDING_SELF_SETS
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| v.checked_sub(1))
            .is_ok();
        if was_self_set {
            log::debug!("[clipboard] ignoring self-triggered clipboard update");
            return Ok(());
        }

        // Capture the clipboard owner now, while it still reflects the app that
        // just changed the clipboard content, rather than inside the spawned
        // thread where it could already have changed again.
        let owner_hwnd = unsafe { GetClipboardOwner() }
            .map(|hwnd| hwnd.0 as isize)
            .unwrap_or(0);
        // Handle on a dedicated, throwaway thread instead of `sta_call`'s
        // blocking wait: this callback runs on the *shared* background-window
        // dispatch thread that every other module's events also go through
        // (trash bin, power, shell hook, ...). If clipboard processing ever
        // blocks or is slow, doing it inline here would stall every one of
        // them, not just clipboard, for as long as it takes (or forever).
        spawn_sta("failed to process clipboard content change", move || {
            Self::handle_content_changed(owner_hwnd)
        });
        Ok(())
    }

    fn handle_content_changed(owner_hwnd: isize) -> Result<()> {
        let view = Clipboard::GetContent()?;
        let entry = Self::process_view(&view, owner_hwnd)?;

        let size = entry.content.approx_size();
        if size == 0 {
            log::debug!(
                "[clipboard] skipping empty entry (app={:?})",
                entry.source_app_name
            );
            return Ok(());
        }

        if size > CLIPBOARD_MAX_ENTRY_BYTES {
            log::debug!(
                "[clipboard] skipping entry larger than the configured limit ({size} bytes > {CLIPBOARD_MAX_ENTRY_BYTES})"
            );
            return Ok(());
        }

        let manager = Self::instance();
        let fingerprint = entry.content.fingerprint();

        // Look up and (depending on the outcome) update or insert under a
        // single lock, so that two clipboard changes handled concurrently
        // (each on its own thread, see `on_bg_window_proc`) can't both see
        // "no duplicate" and insert the same content twice.
        let is_new = manager.entries.with_lock(|map| {
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
            Self::evict_if_needed();
        }

        Self::send(ClipboardEvent::EntriesChanged);
        Self::request_persist();
        log::debug!("[clipboard] entries count now = {}", manager.entries.len());
        Ok(())
    }

    /// Evicts the oldest entries until both the entry-count and total-size
    /// limits are satisfied.
    fn evict_if_needed() {
        let manager = Self::instance();
        manager.entries.with_lock(|map| loop {
            let over_count = map.len() > CLIPBOARD_MAX_ENTRIES;
            let total_size: usize = map.values().map(|e| e.content.approx_size()).sum();
            let over_size = total_size > CLIPBOARD_MAX_TOTAL_BYTES;

            if !over_count && !over_size {
                break;
            }

            let Some(oldest_id) = map.values().min_by_key(|e| e.timestamp).map(|e| e.id.clone())
            else {
                break;
            };

            log::debug!(
                "[clipboard] evicting oldest entry {oldest_id} (over_count={over_count}, over_size={over_size})"
            );
            map.remove(&oldest_id);
        });
    }

    fn on_history_enabled_changed(
        _sender: windows_core::Ref<windows_core::IInspectable>,
        _args: windows_core::Ref<windows_core::IInspectable>,
    ) -> windows_core::Result<()> {
        if let Ok(enabled) = Clipboard::IsHistoryEnabled() {
            log::debug!("[clipboard] native IsHistoryEnabled changed -> {enabled}");
            Self::instance()
                .history_enabled
                .store(enabled, Ordering::SeqCst);
            Self::send(ClipboardEvent::EnabledChanged);
        }
        Ok(())
    }

    fn init(&mut self) -> Result<()> {
        self.history_enabled.store(
            Clipboard::IsHistoryEnabled().unwrap_or_default(),
            Ordering::SeqCst,
        );
        log::debug!(
            "[clipboard] init: native IsHistoryEnabled={}",
            self.history_enabled.load(Ordering::SeqCst)
        );

        let store = persistence::load_store();
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

        self.history_enabled_token = Some(Clipboard::HistoryEnabledChanged(&EventHandler::new(
            Self::on_history_enabled_changed,
        ))?);

        subscribe_to_background_window(Self::on_bg_window_proc);

        log::debug!("[clipboard] init done, listening for WM_CLIPBOARDUPDATE");
        Ok(())
    }
}

impl Drop for ClipboardManager {
    fn drop(&mut self) {
        if let Some(token) = self.history_enabled_token.take() {
            Clipboard::RemoveHistoryEnabledChanged(token).log_error();
        }
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

fn get_content_from_view(view: &DataPackageView) -> Result<ClipboardEntryContent> {
    let mut content = ClipboardEntryContent::default();

    if view.Contains(&StandardDataFormats::Text()?)?
        && let Ok(text) = WindowsApi::join_pumped(&view.GetTextAsync()?)
    {
        content.text = Some(text.to_string());
    }

    if view.Contains(&StandardDataFormats::Html()?)?
        && let Ok(html) = WindowsApi::join_pumped(&view.GetHtmlFormatAsync()?)
    {
        content.html = Some(html.to_string());
    }

    if view.Contains(&StandardDataFormats::Rtf()?)?
        && let Ok(rtf) = WindowsApi::join_pumped(&view.GetRtfAsync()?)
    {
        content.rtf = Some(rtf.to_string());
    }

    if view.Contains(&StandardDataFormats::Bitmap()?)?
        && let Ok(stream_ref) = WindowsApi::join_pumped(&view.GetBitmapAsync()?)
    {
        let stream = WindowsApi::join_pumped(&stream_ref.OpenReadAsync()?)?;
        let image = WindowsApi::stream_to_dynamic_image(stream)?;
        let data = WindowsApi::dynamic_image_to_webp_base64(image)?;

        content.bitmap = Some(data);
    }

    if view.Contains(&StandardDataFormats::StorageItems()?)?
        && let Ok(list) = WindowsApi::join_pumped(&view.GetStorageItemsAsync()?)
    {
        let mut data = Vec::new();
        for item in list {
            let path = item.Path()?.to_string();
            data.push(path);
        }

        content.files = Some(data);
    }

    if view.Contains(&StandardDataFormats::ApplicationLink()?)?
        && let Ok(uri) = WindowsApi::join_pumped(&view.GetApplicationLinkAsync()?)
    {
        content.application_link = Some(uri.DisplayUri()?.to_string());
    }

    if view.Contains(&StandardDataFormats::WebLink()?)?
        && let Ok(uri) = WindowsApi::join_pumped(&view.GetWebLinkAsync()?)
    {
        content.web_link = Some(uri.DisplayUri()?.to_string());
    }

    Ok(content)
}
