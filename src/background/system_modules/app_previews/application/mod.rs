#[cfg(windows)]
pub(crate) mod windows;

use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
    time::{Duration, Instant},
};

use seelen_core::rect::Rect;
use seelen_core::system_state::{
    LiveWindowPreviewState, UserAppWindowColors, UserAppWindowPreview, WindowPeekStatus,
    WindowPreviewTarget,
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

pub struct AppPreviewsManager {
    platform: Platform,
    previews: SyncHashMap<isize, PreviewEntry>,
    /// One-by-one capture queue, processed by a dedicated thread.
    capture_tx: crossbeam_channel::Sender<isize>,
    pending: TracedMutex<HashSet<isize>>,
}

event_manager!(AppPreviewsManager, PreviewsEvent);

impl AppPreviewsManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<AppPreviewsManager> = LazyLock::new(|| {
            let mut m = AppPreviewsManager::create();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        let (capture_tx, capture_rx) = crossbeam_channel::unbounded::<isize>();
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
        for hwnd in apps.interactable_windows.map(|w| w.hwnd) {
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

    fn register_window(&self, hwnd: isize) {
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
    fn request_capture(&self, hwnd: isize) {
        self.previews.get(&hwnd, |entry| entry.capture.call(()));
    }

    fn queue_capture(&self, hwnd: isize) {
        let inserted = self.pending.lock().insert(hwnd);
        if inserted && self.capture_tx.send(hwnd).is_err() {
            self.pending.lock().remove(&hwnd);
        }
    }

    /// Refresh only visible snapshot fallbacks, at most four times per second.
    pub fn refresh_fallback(&self, hwnd: isize) {
        let mut requested = false;
        self.previews.get(&hwnd, |entry| {
            if entry
                .requested
                .is_none_or(|time| time.elapsed() >= Duration::from_millis(250))
            {
                entry.requested = Some(Instant::now());
                requested = true;
            }
        });
        if requested {
            self.queue_capture(hwnd);
        }
    }

    fn capture(&self, hwnd: isize) -> Result<()> {
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

    pub fn update_live(
        &self,
        owner: isize,
        session: String,
        targets: Vec<WindowPreviewTarget>,
        opacity: u8,
        anchor: Option<Rect>,
    ) -> Result<LiveWindowPreviewState> {
        self.platform
            .update_live(owner, session, targets, opacity, anchor)
    }
    pub fn clear_live(&self, owner: isize, session: &str) {
        self.platform.clear_live(owner, session);
    }
    pub fn begin_peek(
        &self,
        owner: isize,
        hwnd: isize,
        session: String,
        rect: Rect,
    ) -> Result<WindowPeekStatus> {
        self.platform.begin_peek(owner, hwnd, session, rect)
    }
    pub fn poll_peek(&self, owner: isize, session: &str, rect: Option<Rect>) -> WindowPeekStatus {
        self.platform.poll_peek(owner, session, rect)
    }
    pub fn cancel_peek(&self, owner: isize, session: &str) {
        self.platform.cancel_peek(owner, session);
    }
    pub fn commit_peek(&self, owner: isize, session: &str) -> Result<bool> {
        self.platform.commit_peek(owner, session)
    }

    pub fn previews(&self) -> HashMap<isize, UserAppWindowPreview> {
        let mut map = HashMap::new();
        self.previews.for_each(|(k, v)| {
            if let Some(preview) = &v.preview {
                map.insert(*k, preview.clone());
            }
        });
        map
    }

    pub fn colors(&self) -> HashMap<isize, UserAppWindowColors> {
        let mut map = HashMap::new();
        self.previews.for_each(|(k, v)| {
            if let Some(colors) = &v.colors {
                map.insert(*k, colors.clone());
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
