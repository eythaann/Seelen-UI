use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        OnceLock,
    },
};

use seelen_core::system_state::{ClipboardData, ClipboardEntry, ClipboardEntryContent};
use windows::{
    ApplicationModel::DataTransfer::{
        Clipboard, ClipboardHistoryChangedEventArgs, ClipboardHistoryItem,
        ClipboardHistoryItemsResultStatus, DataPackageView, StandardDataFormats,
    },
    Foundation::EventHandler,
    Win32::{
        System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED},
        UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, TranslateMessage, MSG},
    },
};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    utils::lock_free::SyncHashMap,
    windows_api::WindowsApi,
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
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        };
        tx.send(f()).ok();
    });
    rx.recv().map_err(|e| e.to_string())?
}

#[derive(Debug, Clone)]
pub enum ClipboardEvent {
    HistoryChanged,
    EnabledChanged,
}

pub struct ClipboardManager {
    /// Already-processed entries keyed by their Windows-assigned id.
    /// Shared between the STA pump thread (event handler) and sta_call threads (get_data).
    pub entries: SyncHashMap<String, ClipboardEntry>,
    pub history_enabled: AtomicBool,

    history_changed_token: Option<i64>,
    history_enabled_token: Option<i64>,
}

unsafe impl Send for ClipboardManager {}
unsafe impl Sync for ClipboardManager {}

event_manager!(ClipboardManager, ClipboardEvent);

impl ClipboardManager {
    fn new() -> Self {
        Self {
            entries: SyncHashMap::new(),
            history_enabled: AtomicBool::new(false),
            history_changed_token: None,
            history_enabled_token: None,
        }
    }

    pub fn instance() -> &'static Self {
        static MANAGER: OnceLock<ClipboardManager> = OnceLock::new();
        if MANAGER.get().is_none() {
            // Clipboard WinRT APIs require an STA thread. Spawn a dedicated one
            // that initialises COM as STA, registers event listeners, then runs a
            // Windows message pump so that WinRT can dispatch event callbacks.
            let (ready_tx, ready_rx) = std::sync::mpsc::channel::<()>();
            std::thread::spawn(move || {
                unsafe {
                    let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
                };

                MANAGER.get_or_init(|| {
                    let mut m = ClipboardManager::new();
                    m.init().log_error();
                    m
                });

                // Signal that the manager is ready before entering the pump.
                ready_tx.send(()).ok();

                // Pump messages so WinRT delivers HistoryChanged / HistoryEnabledChanged.
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
            });

            ready_rx.recv().ok();
        }
        MANAGER.get().expect("clipboard manager not initialised")
    }

    pub fn get_data(&self) -> ClipboardData {
        let mut history = self.entries.values();
        history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        ClipboardData {
            history,
            is_history_enabled: self.history_enabled.load(Ordering::SeqCst),
        }
    }

    /// Deletes the history entry with the given id.
    pub fn delete_entry(id: &str) -> Result<()> {
        let id = id.to_owned();
        sta_call(move || {
            let result = Clipboard::GetHistoryItemsAsync()?.join()?;
            if result.Status()? != ClipboardHistoryItemsResultStatus::Success {
                return Ok(());
            }
            let items = result.Items()?;
            for item in items {
                if item.Id()? == id {
                    Clipboard::DeleteItemFromHistory(&item)?;
                    return Ok(());
                }
            }
            Ok(())
        })
    }

    /// Removes all entries from the clipboard history.
    pub fn clear_history() -> Result<()> {
        sta_call(|| {
            Clipboard::ClearHistory()?;
            Ok(())
        })
    }

    /// Sets the given history entry as the current clipboard content.
    pub fn set_clipboard_content(id: &str) -> Result<()> {
        let id = id.to_owned();
        sta_call(move || {
            let result = Clipboard::GetHistoryItemsAsync()?.join()?;
            if result.Status()? != ClipboardHistoryItemsResultStatus::Success {
                return Ok(());
            }
            let items = result.Items()?;
            for item in items {
                if item.Id()? == id {
                    Clipboard::SetHistoryItemAsContent(&item)?;
                    return Ok(());
                }
            }
            Ok(())
        })
    }

    /// Processes a single WinRT history item into a [`ClipboardEntry`].
    /// Must be called from an STA-initialised thread.
    fn process_item(item: &ClipboardHistoryItem) -> Result<ClipboardEntry> {
        let id = item.Id()?.to_string();
        let timestamp = item.Timestamp()?.UniversalTime;
        let content_view = item.Content()?;

        let props = content_view.Properties()?;
        let source_app_name = props
            .ApplicationName()
            .ok()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());

        let source_app_logo = props
            .Square30x30Logo()
            .ok()
            .and_then(|logo_ref| WindowsApi::stream_ref_to_webp_base64(logo_ref).ok());

        let content = get_content_from_view(&content_view)?;

        Ok(ClipboardEntry {
            id,
            timestamp,
            source_app_name,
            source_app_logo,
            content,
        })
    }

    fn fetch_history() -> Result<HashMap<String, ClipboardHistoryItem>> {
        let result = Clipboard::GetHistoryItemsAsync()?.join()?;
        if result.Status()? != ClipboardHistoryItemsResultStatus::Success {
            return Err("Failed to read clipboard history".into());
        }

        let items = result.Items()?;
        let mut map = HashMap::new();
        for item in items {
            map.insert(item.Id()?.to_string(), item);
        }

        Ok(map)
    }

    fn on_history_changed(
        _sender: windows_core::Ref<windows_core::IInspectable>,
        _args: windows_core::Ref<ClipboardHistoryChangedEventArgs>,
    ) -> windows_core::Result<()> {
        let items = match Self::fetch_history() {
            Ok(items) => items,
            Err(e) => {
                log::error!("Failed to read clipboard history: {e}");
                return Ok(());
            }
        };

        // Snapshot the old state; keys remaining at the end are removed entries.
        let mut old_state = Self::instance().entries.to_hash_map();
        let mut new_state = HashMap::new();

        for (id, item) in items {
            if let Some(entry) = old_state.remove(&id) {
                // Already known — reuse without re-processing.
                new_state.insert(id, entry);
                continue;
            }

            // New item — process it for the first time.
            match Self::process_item(&item) {
                Ok(entry) => {
                    new_state.insert(id, entry);
                }
                Err(e) => {
                    log::error!("Failed to process clipboard entry: {e}");
                }
            }
        }

        Self::instance().entries.replace(new_state);
        Self::send(ClipboardEvent::HistoryChanged);
        Ok(())
    }

    fn on_history_enabled_changed(
        _sender: windows_core::Ref<windows_core::IInspectable>,
        _args: windows_core::Ref<windows_core::IInspectable>,
    ) -> windows_core::Result<()> {
        if let Ok(enabled) = Clipboard::IsHistoryEnabled() {
            Self::instance()
                .history_enabled
                .store(enabled, Ordering::SeqCst);
            Self::send(ClipboardEvent::EnabledChanged);
        }
        Ok(())
    }

    fn init(&mut self) -> Result<()> {
        self.history_enabled
            .store(Clipboard::IsHistoryEnabled()?, Ordering::SeqCst);

        let items = Self::fetch_history()?;
        for (id, item) in items {
            let entry = Self::process_item(&item)?;
            self.entries.upsert(id, entry);
        }

        self.history_changed_token = Some(Clipboard::HistoryChanged(&EventHandler::new(
            Self::on_history_changed,
        ))?);

        self.history_enabled_token = Some(Clipboard::HistoryEnabledChanged(&EventHandler::new(
            Self::on_history_enabled_changed,
        ))?);

        Ok(())
    }
}

impl Drop for ClipboardManager {
    fn drop(&mut self) {
        if let Some(token) = self.history_changed_token.take() {
            Clipboard::RemoveHistoryChanged(token).log_error();
        }
        if let Some(token) = self.history_enabled_token.take() {
            Clipboard::RemoveHistoryEnabledChanged(token).log_error();
        }
    }
}

fn get_content_from_view(view: &DataPackageView) -> Result<ClipboardEntryContent> {
    let mut content = ClipboardEntryContent::default();

    if view.Contains(&StandardDataFormats::Text()?)? {
        if let Ok(text) = view.GetTextAsync()?.join() {
            content.text = Some(text.to_string());
        }
    }

    if view.Contains(&StandardDataFormats::Html()?)? {
        if let Ok(html) = view.GetHtmlFormatAsync()?.join() {
            content.html = Some(html.to_string());
        }
    }

    if view.Contains(&StandardDataFormats::Rtf()?)? {
        if let Ok(rtf) = view.GetRtfAsync()?.join() {
            content.rtf = Some(rtf.to_string());
        }
    }

    if view.Contains(&StandardDataFormats::Bitmap()?)? {
        if let Ok(stream_ref) = view.GetBitmapAsync()?.join() {
            let stream = stream_ref.OpenReadAsync()?.join()?;
            let image = WindowsApi::stream_to_dynamic_image(stream)?;
            let data = WindowsApi::dynamic_image_to_webp_base64(image)?;

            content.bitmap = Some(data);
        }
    }

    if view.Contains(&StandardDataFormats::StorageItems()?)? {
        if let Ok(list) = view.GetStorageItemsAsync()?.join() {
            let mut data = Vec::new();
            for item in list {
                let name = item.Name()?.to_string();
                data.push(name);
            }

            content.files = Some(data);
        }
    }

    if view.Contains(&StandardDataFormats::ApplicationLink()?)? {
        if let Ok(uri) = view.GetApplicationLinkAsync()?.join() {
            content.application_link = Some(uri.DisplayUri()?.to_string());
        }
    }

    if view.Contains(&StandardDataFormats::WebLink()?)? {
        if let Ok(uri) = view.GetWebLinkAsync()?.join() {
            content.web_link = Some(uri.DisplayUri()?.to_string());
        }
    }

    Ok(content)
}
