use std::{
    collections::{HashMap, VecDeque},
    sync::LazyLock,
    time::{Duration, Instant},
};

use seelen_core::{
    rect::Rect,
    system_state::{LiveWindowPreviewState, WindowPreviewTarget},
};
use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Dwm::*,
    UI::WindowsAndMessaging::{GetCursorPos, GetWindowRect, IsWindow, IsWindowVisible},
};

use crate::{error::Result, utils::lock_free::TracedMutex, windows_api::WindowsApi};

const LEASE_TTL: Duration = Duration::from_secs(2);
const CANCEL_TTL: Duration = Duration::from_secs(30);
const MAX_TARGETS: usize = 16;

/// DWM owns frame updates. Only layout/opacity and the lifetime lease cross the webview boundary.
pub struct LivePreviewManager {
    state: TracedMutex<LivePreviewState>,
}

impl LivePreviewManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<LivePreviewManager> = LazyLock::new(|| {
            std::thread::Builder::new()
                .name("LivePreviewCleanup".into())
                .spawn(|| {
                    loop {
                        std::thread::sleep(Duration::from_millis(150));
                        LivePreviewManager::instance().cleanup();
                    }
                })
                .expect("live preview cleanup");
            LivePreviewManager {
                state: TracedMutex::new(LivePreviewState::default()),
            }
        });
        &MANAGER
    }

    pub fn update(
        &self,
        owner: isize,
        session: String,
        targets: Vec<WindowPreviewTarget>,
        opacity: u8,
        anchor: Option<Rect>,
    ) -> Result<LiveWindowPreviewState> {
        if session.is_empty() || session.len() > 128 || targets.len() > MAX_TARGETS {
            return Err("Invalid live preview request".into());
        }
        let visible = unsafe { IsWindowVisible(HWND(owner as _)) }.as_bool();
        let mut state = self.state.lock();
        state.prune();
        if state
            .cancelled
            .iter()
            .any(|(window, token, _)| *window == owner && token == &session)
        {
            return Ok(LiveWindowPreviewState::default());
        }
        if !visible {
            state.owners.remove(&owner);
            return Ok(LiveWindowPreviewState::default());
        }
        let entry = state
            .owners
            .entry(owner)
            .or_insert_with(|| PreviewOwner::new(session.clone()));
        if entry.session != session {
            *entry = PreviewOwner::new(session);
        }
        entry.refreshed = Instant::now();
        entry
            .thumbnails
            .retain(|hwnd, _| targets.iter().any(|target| target.hwnd == *hwnd));
        let mut live = Vec::new();
        for target in targets {
            if !valid_rect(&target.rect) || target.hwnd == owner {
                continue;
            }
            if let std::collections::hash_map::Entry::Vacant(slot) =
                entry.thumbnails.entry(target.hwnd)
            {
                match LiveThumbnail::register(owner, target.hwnd) {
                    Ok(thumbnail) => {
                        slot.insert(thumbnail);
                    }
                    Err(_) => {
                        continue;
                    }
                }
            }
            let thumbnail = entry
                .thumbnails
                .get_mut(&target.hwnd)
                .expect("registered thumbnail");
            if thumbnail.update(&target.rect, opacity).is_ok() {
                live.push(target.hwnd);
            } else {
                entry.thumbnails.remove(&target.hwnd);
            }
        }
        Ok(LiveWindowPreviewState {
            live,
            visible,
            pointer_inside: pointer_inside_bridge(owner, anchor.as_ref()),
        })
    }

    pub fn clear(&self, owner: isize, session: &str) {
        let mut state = self.state.lock();
        state.prune();
        state
            .cancelled
            .push_back((owner, session.to_owned(), Instant::now()));
        if state
            .owners
            .get(&owner)
            .is_some_and(|entry| entry.session == session)
        {
            state.owners.remove(&owner);
        }
    }

    pub fn clear_all(&self) {
        self.state.lock().owners.clear();
    }

    fn cleanup(&self) {
        let mut state = self.state.lock();
        state.owners.retain(|owner, entry| {
            entry.refreshed.elapsed() < LEASE_TTL
                && unsafe { IsWindowVisible(HWND(*owner as _)) }.as_bool()
        });
        state.prune();
    }
}

#[derive(Default)]
struct LivePreviewState {
    owners: HashMap<isize, PreviewOwner>,
    cancelled: VecDeque<(isize, String, Instant)>,
}
impl LivePreviewState {
    fn prune(&mut self) {
        while self
            .cancelled
            .front()
            .is_some_and(|(_, _, time)| time.elapsed() > CANCEL_TTL)
            || self.cancelled.len() >= 128
        {
            self.cancelled.pop_front();
        }
    }
}

struct PreviewOwner {
    session: String,
    refreshed: Instant,
    thumbnails: HashMap<isize, LiveThumbnail>,
}
impl PreviewOwner {
    fn new(session: String) -> Self {
        Self {
            session,
            refreshed: Instant::now(),
            thumbnails: HashMap::new(),
        }
    }
}

struct LiveThumbnail {
    handle: isize,
    source: isize,
    last: Option<(Rect, Rect, u8)>,
}
impl LiveThumbnail {
    fn register(owner: isize, source: isize) -> Result<Self> {
        let handle = unsafe { DwmRegisterThumbnail(HWND(owner as _), HWND(source as _))? };
        Ok(Self {
            handle,
            source,
            last: None,
        })
    }
    fn update(&mut self, destination: &Rect, opacity: u8) -> Result<()> {
        if !unsafe { IsWindow(Some(HWND(self.source as _))) }.as_bool() {
            return Err("Preview source has closed".into());
        }
        let size = unsafe { DwmQueryThumbnailSourceSize(self.handle)? };
        let bounds = source_bounds(self.source, size.cx, size.cy);
        let source = cover_crop(&bounds, destination).ok_or("Empty preview surface")?;
        let key = (source.clone(), destination.clone(), opacity);
        if self.last.as_ref() == Some(&key) {
            return Ok(());
        }
        let properties = DWM_THUMBNAIL_PROPERTIES {
            dwFlags: DWM_TNP_RECTDESTINATION
                | DWM_TNP_RECTSOURCE
                | DWM_TNP_OPACITY
                | DWM_TNP_VISIBLE
                | DWM_TNP_SOURCECLIENTAREAONLY,
            rcDestination: native_rect(destination),
            rcSource: native_rect(&source),
            opacity,
            fVisible: (opacity > 0).into(),
            fSourceClientAreaOnly: false.into(),
        };
        unsafe {
            DwmUpdateThumbnailProperties(self.handle, &properties)?;
        }
        self.last = Some(key);
        Ok(())
    }
}
impl Drop for LiveThumbnail {
    fn drop(&mut self) {
        unsafe {
            let _ = DwmUnregisterThumbnail(self.handle);
        }
    }
}

fn valid_rect(rect: &Rect) -> bool {
    rect.right > rect.left
        && rect.bottom > rect.top
        && [rect.left, rect.top, rect.right, rect.bottom]
            .iter()
            .all(|value| value.unsigned_abs() <= 32768)
}
fn native_rect(rect: &Rect) -> RECT {
    RECT {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    }
}
fn source_bounds(source: isize, width: i32, height: i32) -> Rect {
    let full = Rect {
        left: 0,
        top: 0,
        right: width,
        bottom: height,
    };
    let Ok(shadow) = WindowsApi::shadow_rect(HWND(source as _)) else {
        return full;
    };
    let inset = Rect {
        left: shadow.left.abs(),
        top: shadow.top.abs(),
        right: width - shadow.right.abs(),
        bottom: height - shadow.bottom.abs(),
    };
    if valid_rect(&inset) { inset } else { full }
}

/// Match the existing top-aligned object-fit:cover layout, without stretching or letterboxing.
fn cover_crop(source: &Rect, destination: &Rect) -> Option<Rect> {
    if !valid_rect(source) || !valid_rect(destination) {
        return None;
    }
    let width = source.right - source.left;
    let height = source.bottom - source.top;
    let dw = destination.right - destination.left;
    let dh = destination.bottom - destination.top;
    if i64::from(width) * i64::from(dh) > i64::from(height) * i64::from(dw) {
        let cropped = (i64::from(height) * i64::from(dw) / i64::from(dh)).max(1) as i32;
        let left = source.left + (width - cropped) / 2;
        Some(Rect {
            left,
            top: source.top,
            right: left + cropped,
            bottom: source.bottom,
        })
    } else {
        let cropped = (i64::from(width) * i64::from(dh) / i64::from(dw)).max(1) as i32;
        Some(Rect {
            left: source.left,
            top: source.top,
            right: source.right,
            bottom: source.top + cropped,
        })
    }
}
fn pointer_inside_bridge(owner: isize, anchor: Option<&Rect>) -> bool {
    let mut point = Default::default();
    let mut popup = RECT::default();
    if unsafe { GetCursorPos(&mut point) }.is_err()
        || unsafe { GetWindowRect(HWND(owner as _), &mut popup) }.is_err()
    {
        return false;
    }
    let left = anchor.map_or(popup.left, |rect| popup.left.min(rect.left));
    let top = anchor.map_or(popup.top, |rect| popup.top.min(rect.top));
    let right = anchor.map_or(popup.right, |rect| popup.right.max(rect.right));
    let bottom = anchor.map_or(popup.bottom, |rect| popup.bottom.max(rect.bottom));
    point.x >= left && point.x <= right && point.y >= top && point.y <= bottom
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::{Win32::UI::WindowsAndMessaging::*, core::w};

    // All fixture windows are offscreen and never activated. DWM updates their surfaces natively.
    struct TestWindow(HWND);
    impl TestWindow {
        fn new() -> Self {
            unsafe {
                let hwnd = CreateWindowExW(
                    WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
                    w!("STATIC"),
                    w!("Seelen live preview test"),
                    WS_OVERLAPPEDWINDOW,
                    -30000,
                    -30000,
                    640,
                    480,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap();
                let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                Self(hwnd)
            }
        }
        fn address(&self) -> isize {
            self.0.0 as isize
        }
    }
    impl Drop for TestWindow {
        fn drop(&mut self) {
            unsafe {
                let _ = DestroyWindow(self.0);
            }
        }
    }

    #[test]
    fn crop_fills_portrait_landscape_ultrawide_and_inset_sources() {
        let destination = Rect {
            left: 10,
            top: 20,
            right: 266,
            bottom: 164,
        };
        for source in [
            Rect {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1080,
            },
            Rect {
                left: 0,
                top: 0,
                right: 500,
                bottom: 1200,
            },
            Rect {
                left: 8,
                top: 8,
                right: 3432,
                bottom: 1432,
            },
        ] {
            let crop = cover_crop(&source, &destination).unwrap();
            assert!(
                crop.left >= source.left
                    && crop.top == source.top
                    && crop.right <= source.right
                    && crop.bottom <= source.bottom
            );
            let width = crop.right - crop.left;
            let height = crop.bottom - crop.top;
            assert!((width * 144 - height * 256).abs() <= 256);
        }
        assert!(cover_crop(&Rect::default(), &destination).is_none());
        assert!(cover_crop(&destination, &Rect::default()).is_none());
    }

    #[test]
    #[ignore = "requires an interactive Windows desktop with DWM"]
    fn live_surface_is_reused_and_unregistered_on_drop() {
        let owner = TestWindow::new();
        let source = TestWindow::new();
        let foreground = unsafe { GetForegroundWindow() };
        let mut thumbnail = LiveThumbnail::register(owner.address(), source.address()).unwrap();
        let handle = thumbnail.handle;
        let rect = Rect {
            left: 10,
            top: 40,
            right: 266,
            bottom: 184,
        };
        thumbnail.update(&rect, 255).unwrap();
        thumbnail.update(&rect, 100).unwrap();
        assert_eq!(thumbnail.handle, handle);
        assert_eq!(unsafe { GetForegroundWindow() }, foreground);
        assert!(unsafe { DwmQueryThumbnailSourceSize(handle) }.is_ok());
        drop(thumbnail);
        assert!(unsafe { DwmQueryThumbnailSourceSize(handle) }.is_err());
    }

    #[test]
    #[ignore = "requires an interactive Windows desktop with DWM"]
    fn cancellation_rejects_late_frames_and_cannot_clear_a_new_session() {
        let owner = TestWindow::new();
        let source = TestWindow::new();
        let manager = LivePreviewManager {
            state: TracedMutex::new(LivePreviewState::default()),
        };
        let targets = vec![WindowPreviewTarget {
            hwnd: source.address(),
            rect: Rect {
                left: 10,
                top: 40,
                right: 266,
                bottom: 184,
            },
        }];
        manager.clear(owner.address(), "old");
        assert!(
            manager
                .update(owner.address(), "old".into(), targets.clone(), 255, None)
                .unwrap()
                .live
                .is_empty()
        );
        assert_eq!(
            manager
                .update(owner.address(), "new".into(), targets, 255, None)
                .unwrap()
                .live,
            vec![source.address()]
        );
        manager.clear(owner.address(), "old");
        assert!(manager.state.lock().owners.contains_key(&owner.address()));
        manager.clear(owner.address(), "new");
        assert!(manager.state.lock().owners.is_empty());
    }

    #[test]
    #[ignore = "requires an interactive Windows desktop with DWM"]
    fn hidden_destination_or_expired_heartbeat_releases_the_surface() {
        let owner = TestWindow::new();
        let source = TestWindow::new();
        let manager = LivePreviewManager {
            state: TracedMutex::new(LivePreviewState::default()),
        };
        let targets = vec![WindowPreviewTarget {
            hwnd: source.address(),
            rect: Rect {
                left: 10,
                top: 40,
                right: 266,
                bottom: 184,
            },
        }];
        assert_eq!(
            manager
                .update(owner.address(), "first".into(), targets.clone(), 255, None)
                .unwrap()
                .live
                .len(),
            1
        );
        unsafe {
            let _ = ShowWindow(owner.0, SW_HIDE);
        }
        manager.cleanup();
        assert!(manager.state.lock().owners.is_empty());
        unsafe {
            let _ = ShowWindow(owner.0, SW_SHOWNOACTIVATE);
        }
        assert_eq!(
            manager
                .update(owner.address(), "second".into(), targets, 255, None)
                .unwrap()
                .live
                .len(),
            1
        );
        manager
            .state
            .lock()
            .owners
            .get_mut(&owner.address())
            .unwrap()
            .refreshed = Instant::now() - LEASE_TTL;
        manager.cleanup();
        assert!(manager.state.lock().owners.is_empty());
    }
}
