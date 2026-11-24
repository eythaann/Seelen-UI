#[cfg(windows)]
mod windows;

use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
    time::{Duration, Instant},
};

use seelen_core::system_state::{
    LiveWindowPreviewState, UserAppWindowColors, UserAppWindowPreview, WindowHandle,
    WindowIdentity, WindowPreviewTarget,
};
use slu_utils::{Debounce, debounce};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    system_modules::apps::{UserAppWinEvent, UserAppsManager},
    utils::lock_free::{SyncHashMap, TracedMutex},
};

use super::domain::{
    AppPreviewsSystem, PreviewsEvent, PreviewsSysEvent, encode_preview, image_hash,
    sample_edge_colors, shrink_capture,
};

#[cfg(windows)]
type Platform = windows::WindowsAppPreviews;

/// Captures of the same window are debounced by this interval.
const CAPTURE_DEBOUNCE: Duration = Duration::from_millis(200);
/// A restored window takes a moment to draw its content.
const RESTORED_CAPTURE_DELAY: Duration = Duration::from_millis(300);
/// Snapshot fallbacks (windows DWM could not compose live) are refreshed at most this often:
/// each capture is encoded and broadcast to every webview.
const FALLBACK_REFRESH: Duration = Duration::from_secs(1);

static MANAGER: OnceLock<AppPreviewsManager> = OnceLock::new();

pub struct AppPreviewsManager {
    platform: Platform,
    previews: SyncHashMap<WindowHandle, PreviewEntry>,
    /// One-by-one capture queue, processed by a dedicated thread.
    capture_tx: crossbeam_channel::Sender<WindowHandle>,
    pending: TracedMutex<HashSet<WindowHandle>>,
}

event_manager!(AppPreviewsManager, PreviewsEvent);

impl AppPreviewsManager {
    pub fn instance() -> &'static Self {
        MANAGER.get_or_init(|| {
            let mut m = AppPreviewsManager::create();
            m.init().log_error();
            m
        })
    }

    /// `None` until something requested previews: shutdown must not spin the manager up.
    pub fn get_if_initialized() -> Option<&'static Self> {
        MANAGER.get()
    }

    fn create() -> Self {
        let (capture_tx, capture_rx) = crossbeam_channel::unbounded::<WindowHandle>();
        std::thread::Builder::new()
            .name("win-capture-queue".into())
            .spawn(move || {
                for hwnd in capture_rx {
                    let manager = Self::instance();
                    if crate::platform::session::is_interactive() {
                        manager.capture(hwnd).log_error();
                    }
                    manager.pending.lock().remove(&hwnd);
                }
            })
            .log_error();

        Self {
            platform: Platform::create(),
            previews: SyncHashMap::new(),
            capture_tx,
            pending: TracedMutex::new(HashSet::new()),
        }
    }

    fn init(&mut self) -> Result<()> {
        let apps = UserAppsManager::instance();
        for hwnd in apps.interactable_windows.map(|w| w.hwnd.hwnd) {
            self.register_window(hwnd);
        }

        UserAppsManager::subscribe(|event| {
            let manager = Self::instance();
            match event {
                UserAppWinEvent::Added(hwnd) => manager.register_window(hwnd),
                UserAppWinEvent::Updated(_) => {}
                UserAppWinEvent::Removed(hwnd) => {
                    if manager.previews.remove(&hwnd).is_some() {
                        Self::send(PreviewsEvent::Cleaned { hwnd });
                    }
                }
            }
        });

        self.platform.init(|event| {
            let manager = Self::instance();
            match event {
                PreviewsSysEvent::WindowChanged { hwnd } => manager.request_capture(hwnd),
                PreviewsSysEvent::WindowRestored { hwnd } => {
                    std::thread::spawn(move || {
                        std::thread::sleep(RESTORED_CAPTURE_DELAY);
                        Self::instance().request_capture(hwnd);
                    });
                }
            }
        })
    }

    fn register_window(&self, hwnd: WindowHandle) {
        let capture = debounce(
            move |_| {
                Self::instance().queue_capture(hwnd);
            },
            CAPTURE_DEBOUNCE,
        );
        self.previews.upsert(
            hwnd,
            PreviewEntry {
                preview: None,
                colors: None,
                capture,
                requested: None,
            },
        );
        self.request_capture(hwnd);
    }

    /// Only the registered (interactable) windows are captured.
    fn request_capture(&self, hwnd: WindowHandle) {
        self.previews.get(&hwnd, |entry| entry.capture.call(()));
    }

    fn queue_capture(&self, hwnd: WindowHandle) {
        let inserted = self.pending.lock().insert(hwnd);
        if inserted && self.capture_tx.send(hwnd).is_err() {
            self.pending.lock().remove(&hwnd);
        }
    }

    /// Refresh only visible snapshot fallbacks, at most once per [`FALLBACK_REFRESH`].
    pub fn refresh_fallback(&self, hwnd: WindowHandle) {
        let mut requested = false;
        self.previews.get(&hwnd, |entry| {
            if entry
                .requested
                .is_none_or(|time| time.elapsed() >= FALLBACK_REFRESH)
            {
                entry.requested = Some(Instant::now());
                requested = true;
            }
        });
        if requested {
            self.queue_capture(hwnd);
        }
    }

    fn capture(&self, hwnd: WindowHandle) -> Result<()> {
        let Some(image) = self.platform.capture(hwnd)? else {
            return Ok(());
        };
        log::trace!("captured window ({hwnd:x})");

        let image = shrink_capture(image);
        let colors = sample_edge_colors(&image);
        let hash = image_hash(&image);

        let unchanged = self
            .previews
            .get(&hwnd, |entry| {
                entry.preview.as_ref().is_some_and(|p| p.hash == hash)
            })
            .unwrap_or(false);

        if !unchanged {
            let preview = encode_preview(image, hash)?;
            self.previews
                .get(&hwnd, |entry| entry.preview = Some(preview));
            Self::send(PreviewsEvent::Captured { hwnd });
        }

        self.previews
            .get(&hwnd, |entry| entry.colors = Some(colors));
        Self::send(PreviewsEvent::ColorsUpdated { hwnd });
        Ok(())
    }

    pub fn clear_native_previews(&self) {
        self.platform.clear_native_previews();
    }

    pub fn update_live(
        &self,
        owner: WindowHandle,
        session: String,
        targets: Vec<WindowPreviewTarget>,
        opacity: u8,
    ) -> Result<LiveWindowPreviewState> {
        self.platform.update_live(owner, session, targets, opacity)
    }
    pub fn clear_live(&self, owner: WindowHandle, session: &str) {
        self.platform.clear_live(owner, session);
    }

    pub fn previews(&self) -> HashMap<WindowIdentity, UserAppWindowPreview> {
        let identities = interactable_identities();
        let mut map = HashMap::new();
        self.previews.for_each(|(k, v)| {
            if let (Some(id), Some(preview)) = (identities.get(k), &v.preview) {
                map.insert(*id, preview.clone());
            }
        });
        map
    }

    pub fn colors(&self) -> HashMap<WindowIdentity, UserAppWindowColors> {
        let identities = interactable_identities();
        let mut map = HashMap::new();
        self.previews.for_each(|(k, v)| {
            if let (Some(id), Some(colors)) = (identities.get(k), &v.colors) {
                map.insert(*id, colors.clone());
            }
        });
        map
    }
}

struct PreviewEntry {
    preview: Option<UserAppWindowPreview>,
    colors: Option<UserAppWindowColors>,
    capture: Debounce<()>,
    requested: Option<Instant>,
}

/// Identities of the tracked app windows by handle, taken while they were alive: a preview can
/// outlive its window until it is cleaned, and a destroyed window can't be queried anymore.
fn interactable_identities() -> HashMap<WindowHandle, WindowIdentity> {
    UserAppsManager::instance()
        .interactable_windows
        .map(|w| (w.hwnd.hwnd, w.hwnd))
        .into_iter()
        .collect()
}
