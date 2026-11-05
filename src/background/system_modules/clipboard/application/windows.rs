use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use seelen_core::system_state::ClipboardEntryContent;
use windows::{
    ApplicationModel::DataTransfer::{
        Clipboard, DataPackage, DataPackageOperation, DataPackageView, StandardDataFormats,
    },
    Foundation::{EventHandler, Uri},
    Security::Cryptography::{CryptographicBuffer, DataProtection::DataProtectionProvider},
    Storage::{IStorageItem, StorageFile, StorageFolder},
    Win32::{
        System::DataExchange::GetClipboardOwner,
        UI::WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, MSG, TranslateMessage, WM_CLIPBOARDUPDATE,
        },
    },
    core::HSTRING,
};
use windows_core::Interface;

use crate::{
    error::{Result, ResultLogExt},
    platform::window::{Window, WindowsWindowExt},
    system_modules::clipboard::domain::{
        CapturedContent, ClipboardStore, ClipboardSysEvent, ClipboardSystem,
    },
    utils::constants::SEELEN_COMMON,
    windows_api::{Com, WindowsApi, event_window::subscribe_to_background_window, input::Keyboard},
};

/// Descriptor scoping protected data to the current Windows user account,
/// on this machine only (equivalent to DPAPI's per-user scope).
const PROTECTION_DESCRIPTOR: &str = "LOCAL=user";

/// Counts clipboard writes we ourselves originated (via [`WindowsClipboard::set_content`])
/// that haven't yet been observed via `WM_CLIPBOARDUPDATE`. Incremented right
/// before we call `Clipboard::SetContent`, decremented when the resulting
/// notification arrives.
///
/// This can't be done by checking the clipboard owner window instead, because
/// restoring an entry can be triggered by our own API/CLI without any of our
/// windows being focused (or existing at all) at that moment, so there's no
/// reliable "our window owns the clipboard" signal to check against.
static PENDING_SELF_SETS: AtomicU32 = AtomicU32::new(0);

type EventCallback = Arc<dyn Fn(ClipboardSysEvent) + Send + Sync>;

/// The clipboard WinRT APIs require an STA thread, so every call runs on one.
pub struct WindowsClipboard {
    history_enabled_token: Option<i64>,
}

impl ClipboardSystem for WindowsClipboard {
    fn create() -> Self {
        Self {
            history_enabled_token: None,
        }
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(ClipboardSysEvent) + Send + Sync + 'static,
    {
        let on_event: EventCallback = Arc::new(on_event);
        self.history_enabled_token = Some(spawn_history_listener(on_event.clone())?);

        // WinRT's `Clipboard.ContentChanged` event does not reliably fire for
        // unpackaged Win32 apps like this one, so we instead rely on the classic
        // `WM_CLIPBOARDUPDATE` notification (see `AddClipboardFormatListener` in
        // `windows_api::event_window`), which works regardless of package identity.
        subscribe_to_background_window(move |msg, _w_param, _l_param| {
            if msg == WM_CLIPBOARDUPDATE {
                on_clipboard_update(on_event.clone());
            }
            Ok(())
        });
        log::debug!("[clipboard] init done, listening for WM_CLIPBOARDUPDATE");
        Ok(())
    }

    fn is_history_enabled(&self) -> bool {
        sta_call(|| Ok(Clipboard::IsHistoryEnabled()?)).unwrap_or_default()
    }

    fn set_content(&self, content: &ClipboardEntryContent, plain: bool) -> Result<()> {
        let content = content.clone();
        sta_call(move || {
            let package = data_package_from_content(&content, plain)?;
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
        })
    }

    fn paste(&self) -> Result<()> {
        Keyboard::new().send_keys("{ctrl}v")
    }

    /// Returns an empty store if there is no store yet, or if it fails to read/decrypt
    /// (corrupted file, moved from another user/machine, etc).
    fn load_store(&self) -> ClipboardStore {
        let path = SEELEN_COMMON.clipboard_history_path();
        if !path.exists() {
            return ClipboardStore::default();
        }

        let load = move || -> Result<ClipboardStore> {
            let encrypted = std::fs::read(path)?;
            let json = decrypt(&encrypted)?;
            Ok(serde_json::from_slice(&json)?)
        };

        match sta_call(load) {
            Ok(store) => store,
            Err(e) => {
                log::error!("Failed to load clipboard history store, starting empty: {e}");
                ClipboardStore::default()
            }
        }
    }

    fn save_store(&self, store: &ClipboardStore) -> Result<()> {
        let json = serde_json::to_vec(store)?;
        let encrypted = sta_call(move || encrypt(&json))?;
        std::fs::write(SEELEN_COMMON.clipboard_history_path(), encrypted)?;
        Ok(())
    }
}

impl Drop for WindowsClipboard {
    fn drop(&mut self) {
        if let Some(token) = self.history_enabled_token.take() {
            Clipboard::RemoveHistoryEnabledChanged(token).log_error();
        }
    }
}

/// Spawns the STA thread listening `HistoryEnabledChanged`: it registers the handler and then
/// pumps messages, so WinRT can dispatch the event. Returns the handler token.
fn spawn_history_listener(on_event: EventCallback) -> Result<i64> {
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<i64>>();
    std::thread::spawn(move || {
        let _ = Com::run_with_context(|| {
            let token = Clipboard::HistoryEnabledChanged(&EventHandler::new(move |_, _| {
                if let Ok(enabled) = Clipboard::IsHistoryEnabled() {
                    log::debug!("[clipboard] native IsHistoryEnabled changed -> {enabled}");
                    on_event(ClipboardSysEvent::HistoryEnabledChanged { enabled });
                }
                Ok(())
            }));
            let registered = token.is_ok();
            ready_tx.send(token.map_err(Into::into)).ok();
            if !registered {
                return Ok(());
            }

            let mut msg = MSG::default();
            while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
                unsafe {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
            Ok(())
        });
    });
    ready_rx.recv().map_err(|e| e.to_string())?
}

/// Called for every `WM_CLIPBOARDUPDATE` received by the shared background window.
fn on_clipboard_update(on_event: EventCallback) {
    log::debug!("[clipboard] WM_CLIPBOARDUPDATE received");

    // Consume one self-set credit, if any, instead of processing this as
    // an externally-originated change. This is what stops restoring an
    // entry (`set_content`) from being re-captured as a brand
    // new entry, which for bitmaps also caused a decode/re-encode loop
    // that degraded the image a little more on every restore.
    let was_self_set = PENDING_SELF_SETS
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| v.checked_sub(1))
        .is_ok();
    if was_self_set {
        log::debug!("[clipboard] ignoring self-triggered clipboard update");
        return;
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
        let view = Clipboard::GetContent()?;
        let capture = capture_view(&view, owner_hwnd)?;
        on_event(ClipboardSysEvent::ContentCaptured {
            capture: Box::new(capture),
        });
        Ok(())
    });
}

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
/// caller, just logs `context` on failure.
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

/// Rebuilds a `DataPackage` from what we have stored for an entry.
fn data_package_from_content(content: &ClipboardEntryContent, plain: bool) -> Result<DataPackage> {
    let package = DataPackage::new()?;
    package.SetRequestedOperation(DataPackageOperation::Copy)?;

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
            items.push(Some(storage_item_from_path(path)?));
        }
        let items = windows_collections::IIterable::<IStorageItem>::from(items);
        package.SetStorageItems(&items, false)?;
        set_something = true;
    }

    if !set_something {
        return Err("This entry has no restorable content".into());
    }
    Ok(package)
}

/// Resolves a filesystem path back into an `IStorageItem`, for restoring
/// file/folder clipboard entries.
fn storage_item_from_path(path: &str) -> Result<IStorageItem> {
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

/// Reads the clipboard view and the app it came from. `owner_hwnd` is the clipboard owner
/// window (via `GetClipboardOwner`) captured at the time of the clipboard change.
/// Must be called from an STA-initialised thread.
fn capture_view(view: &DataPackageView, owner_hwnd: isize) -> Result<CapturedContent> {
    let (source_app_name, source_app_umid, source_app_path) = if owner_hwnd != 0 {
        let window = Window::from(owner_hwnd);
        let name = window.app_display_name().ok().filter(|s| !s.is_empty());
        let umid = window.app_user_model_id().map(|umid| umid.to_string());
        let path = window
            .exe_path()
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

    Ok(CapturedContent {
        content: content_from_view(view)?,
        source_app_name,
        source_app_umid,
        source_app_path,
    })
}

fn content_from_view(view: &DataPackageView) -> Result<ClipboardEntryContent> {
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
        content.bitmap = Some(WindowsApi::dynamic_image_to_webp_base64(image)?);
    }

    if view.Contains(&StandardDataFormats::StorageItems()?)?
        && let Ok(list) = WindowsApi::join_pumped(&view.GetStorageItemsAsync()?)
    {
        let mut data = Vec::new();
        for item in list {
            data.push(item.Path()?.to_string());
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

/// Encrypts `plaintext` so that only the current user, on this machine, can
/// decrypt it back (via [`decrypt`]). Must be called from an STA thread.
fn encrypt(plaintext: &[u8]) -> Result<Vec<u8>> {
    let provider =
        DataProtectionProvider::CreateOverloadExplicit(&HSTRING::from(PROTECTION_DESCRIPTOR))?;
    let input = CryptographicBuffer::CreateFromByteArray(plaintext)?;
    let protected = provider.ProtectAsync(&input)?.join()?;

    let mut output = windows_core::Array::<u8>::new();
    CryptographicBuffer::CopyToByteArray(&protected, &mut output)?;
    Ok(output.to_vec())
}

/// Decrypts data previously produced by [`encrypt`]. Must be called from an STA thread.
fn decrypt(ciphertext: &[u8]) -> Result<Vec<u8>> {
    let provider = DataProtectionProvider::new()?;
    let input = CryptographicBuffer::CreateFromByteArray(ciphertext)?;
    let plain = provider.UnprotectAsync(&input)?.join()?;

    let mut output = windows_core::Array::<u8>::new();
    CryptographicBuffer::CopyToByteArray(&plain, &mut output)?;
    Ok(output.to_vec())
}
