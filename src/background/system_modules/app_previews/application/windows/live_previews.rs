use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

use seelen_core::{
    rect::Rect,
    system_state::{LiveWindowPreviewState, WindowHandle, WindowPreviewTarget},
};
use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Dwm::*,
    UI::WindowsAndMessaging::{IsWindow, IsWindowVisible},
};

use crate::{error::Result, utils::lock_free::TracedMutex, windows_api::WindowsApi};

const CANCEL_TTL: Duration = Duration::from_secs(30);
const MAX_CANCELLED: usize = 128;
const MAX_TARGETS: usize = 16;

/// DWM owns frame updates: only layout/opacity cross the webview boundary, and only when they
/// change. Surfaces live while their destination window is visible; DWM also releases them
/// when that window is destroyed.
pub struct LivePreviewManager {
    state: TracedMutex<LivePreviewState>,
}

impl LivePreviewManager {
    pub fn create() -> Self {
        Self {
            state: TracedMutex::new(LivePreviewState::default()),
        }
    }

    pub fn update(
        &self,
        owner: WindowHandle,
        session: String,
        targets: Vec<WindowPreviewTarget>,
        opacity: u8,
    ) -> Result<LiveWindowPreviewState> {
        if session.is_empty() || session.len() > 128 {
            return Err("Invalid live preview request".into());
        }
        let targets = bounded_targets(targets);
        let mut state = self.state.lock();
        state.prune();
        if state.is_cancelled(owner, &session) {
            return Ok(LiveWindowPreviewState::default());
        }
        // Covers this owner too: a hidden destination keeps no surfaces.
        state.owners.retain(|window, _| is_visible(*window));
        if !is_visible(owner) {
            return Ok(LiveWindowPreviewState::default());
        }

        let entry = state
            .owners
            .entry(owner)
            .or_insert_with(|| PreviewOwner::new(session.clone()));
        if entry.session != session {
            *entry = PreviewOwner::new(session);
        }
        entry
            .thumbnails
            .retain(|hwnd, _| targets.iter().any(|target| target.hwnd.hwnd == *hwnd));

        let mut live = Vec::new();
        for target in targets {
            if !valid_rect(&target.rect) || target.hwnd.hwnd == owner {
                continue;
            }
            let thumbnail = match entry.thumbnails.entry(target.hwnd.hwnd) {
                std::collections::hash_map::Entry::Occupied(slot) => slot.into_mut(),
                std::collections::hash_map::Entry::Vacant(slot) => {
                    match LiveThumbnail::register(owner, target.hwnd.hwnd) {
                        Ok(thumbnail) => slot.insert(thumbnail),
                        Err(_) => continue,
                    }
                }
            };
            if thumbnail
                .update(&target.rect, target.clip.as_ref(), opacity)
                .is_ok()
            {
                live.push(target.hwnd);
            } else {
                entry.thumbnails.remove(&target.hwnd.hwnd);
            }
        }
        Ok(LiveWindowPreviewState {
            live,
            visible: true,
        })
    }

    pub fn clear(&self, owner: WindowHandle, session: &str) {
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
}

#[derive(Default)]
struct LivePreviewState {
    owners: HashMap<WindowHandle, PreviewOwner>,
    cancelled: VecDeque<(WindowHandle, String, Instant)>,
}

impl LivePreviewState {
    fn is_cancelled(&self, owner: WindowHandle, session: &str) -> bool {
        self.cancelled
            .iter()
            .any(|(window, token, _)| *window == owner && token == session)
    }

    fn prune(&mut self) {
        while self
            .cancelled
            .front()
            .is_some_and(|(_, _, time)| time.elapsed() > CANCEL_TTL)
            || self.cancelled.len() >= MAX_CANCELLED
        {
            self.cancelled.pop_front();
        }
    }
}

struct PreviewOwner {
    session: String,
    thumbnails: HashMap<WindowHandle, LiveThumbnail>,
}

impl PreviewOwner {
    fn new(session: String) -> Self {
        Self {
            session,
            thumbnails: HashMap::new(),
        }
    }
}

struct LiveThumbnail {
    handle: isize,
    source: WindowHandle,
    last: Option<(Rect, Rect, u8)>,
}
impl LiveThumbnail {
    fn register(owner: WindowHandle, source: WindowHandle) -> Result<Self> {
        let handle = unsafe { DwmRegisterThumbnail(HWND(owner as _), HWND(source as _))? };
        Ok(Self {
            handle,
            source,
            last: None,
        })
    }
    fn update(&mut self, destination: &Rect, clip: Option<&Rect>, opacity: u8) -> Result<()> {
        if !unsafe { IsWindow(Some(HWND(self.source as _))) }.as_bool() {
            return Err("Preview source has closed".into());
        }
        let size = unsafe { DwmQueryThumbnailSourceSize(self.handle)? };
        let bounds = source_bounds(self.source, size.cx, size.cy);
        let (source, destination) =
            clipped_thumbnail(&bounds, destination, clip).ok_or("Empty preview surface")?;
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
            rcDestination: native_rect(&destination),
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

fn bounded_targets(targets: Vec<WindowPreviewTarget>) -> Vec<WindowPreviewTarget> {
    let mut selected: Vec<WindowPreviewTarget> = Vec::with_capacity(MAX_TARGETS);
    for target in targets {
        if !valid_rect(&target.rect)
            || target
                .clip
                .as_ref()
                .is_some_and(|clip| intersect_rects(&target.rect, clip).is_none())
            || selected.iter().any(|entry| entry.hwnd == target.hwnd)
        {
            continue;
        }
        selected.push(target);
        if selected.len() == MAX_TARGETS {
            break;
        }
    }
    selected
}

fn is_visible(window: WindowHandle) -> bool {
    unsafe { IsWindowVisible(HWND(window as _)) }.as_bool()
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
fn source_bounds(source: WindowHandle, width: i32, height: i32) -> Rect {
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

fn intersect_rects(a: &Rect, b: &Rect) -> Option<Rect> {
    if !valid_rect(a) || !valid_rect(b) {
        return None;
    }
    a.intersection(b)
}

/// Crop the full source first, then map visible destination edges into that crop.
/// Re-running object-fit on the smaller destination would zoom/recenter the picture while scrolling.
fn clipped_thumbnail(bounds: &Rect, full: &Rect, clip: Option<&Rect>) -> Option<(Rect, Rect)> {
    let source = cover_crop(bounds, full)?;
    let visible = match clip {
        Some(clip) => intersect_rects(full, clip)?,
        None => full.clone(),
    };
    if visible == *full {
        return Some((source, visible));
    }
    let map =
        |edge: i32, origin: i32, span: i32, source_origin: i32, source_span: i32, end: bool| {
            let numerator = (i64::from(edge) - i64::from(origin)) * i64::from(source_span);
            let denominator = i64::from(span);
            source_origin
                + ((numerator + if end { denominator - 1 } else { 0 }) / denominator) as i32
        };
    let sw = source.right - source.left;
    let sh = source.bottom - source.top;
    let dw = full.right - full.left;
    let dh = full.bottom - full.top;
    let clipped = Rect {
        left: map(visible.left, full.left, dw, source.left, sw, false),
        top: map(visible.top, full.top, dh, source.top, sh, false),
        right: map(visible.right, full.left, dw, source.left, sw, true),
        bottom: map(visible.bottom, full.top, dh, source.top, sh, true),
    };
    Some((clipped, visible))
}

#[cfg(test)]
mod tests {
    use super::*;
    use seelen_core::system_state::WindowIdentity;

    #[test]
    fn clipped_edges_map_into_the_original_source_without_zooming() {
        let bounds = Rect {
            left: 0,
            top: 0,
            right: 1600,
            bottom: 900,
        };
        let full = Rect {
            left: 10,
            top: 20,
            right: 210,
            bottom: 120,
        };
        for (clip, expected) in [
            (
                Rect {
                    left: 10,
                    top: 70,
                    right: 210,
                    bottom: 120,
                },
                Rect {
                    left: 0,
                    top: 400,
                    right: 1600,
                    bottom: 800,
                },
            ),
            (
                Rect {
                    left: 10,
                    top: 20,
                    right: 210,
                    bottom: 70,
                },
                Rect {
                    left: 0,
                    top: 0,
                    right: 1600,
                    bottom: 400,
                },
            ),
            (
                Rect {
                    left: 60,
                    top: 20,
                    right: 210,
                    bottom: 120,
                },
                Rect {
                    left: 400,
                    top: 0,
                    right: 1600,
                    bottom: 800,
                },
            ),
            (
                Rect {
                    left: 10,
                    top: 20,
                    right: 160,
                    bottom: 120,
                },
                Rect {
                    left: 0,
                    top: 0,
                    right: 1200,
                    bottom: 800,
                },
            ),
        ] {
            let (source, destination) = clipped_thumbnail(&bounds, &full, Some(&clip)).unwrap();
            assert_eq!(source, expected);
            assert_eq!(destination, clip);
        }
        let portrait = Rect {
            left: 0,
            top: 0,
            right: 400,
            bottom: 1200,
        };
        let clip = Rect {
            left: 10,
            top: 45,
            right: 210,
            bottom: 95,
        };
        assert_eq!(
            clipped_thumbnail(&portrait, &full, Some(&clip)).unwrap().0,
            Rect {
                left: 0,
                top: 50,
                right: 400,
                bottom: 150
            }
        );
    }

    #[test]
    fn clipping_preserves_shadow_insets_and_negative_destination_alignment() {
        let bounds = Rect {
            left: 8,
            top: 12,
            right: 1608,
            bottom: 912,
        };
        let full = Rect {
            left: -40,
            top: -20,
            right: 160,
            bottom: 80,
        };
        let clip = Rect {
            left: 0,
            top: 0,
            right: 100,
            bottom: 60,
        };
        let (source, destination) = clipped_thumbnail(&bounds, &full, Some(&clip)).unwrap();
        assert_eq!(destination, clip);
        assert_eq!(
            source,
            Rect {
                left: 328,
                top: 172,
                right: 1128,
                bottom: 652
            }
        );
    }

    #[test]
    fn unclipped_and_empty_surfaces_keep_legacy_behavior_and_rounding_stays_in_bounds() {
        let bounds = Rect {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        };
        let full = Rect {
            left: 17,
            top: 31,
            right: 274,
            bottom: 175,
        };
        let original = cover_crop(&bounds, &full).unwrap();
        assert_eq!(
            clipped_thumbnail(&bounds, &full, None),
            Some((original.clone(), full.clone()))
        );
        assert!(clipped_thumbnail(&bounds, &full, Some(&Rect::default())).is_none());
        assert!(
            clipped_thumbnail(
                &bounds,
                &full,
                Some(&Rect {
                    left: 274,
                    top: 31,
                    right: 300,
                    bottom: 175
                })
            )
            .is_none()
        );
        for offset in [0, 1, 100, 256] {
            let clip = Rect {
                left: full.left + offset,
                top: full.top + 1,
                right: full.left + offset + 1,
                bottom: full.bottom - 1,
            };
            let (source, destination) = clipped_thumbnail(&bounds, &full, Some(&clip)).unwrap();
            assert_eq!(destination, clip);
            assert!(valid_rect(&source));
            assert!(
                source.left >= original.left
                    && source.right <= original.right
                    && source.top >= original.top
                    && source.bottom <= original.bottom
            );
        }
    }

    #[test]
    fn oversized_and_duplicate_groups_are_bounded_without_rejecting_the_group() {
        let rect = Rect {
            left: 10,
            top: 40,
            right: 266,
            bottom: 184,
        };
        let mut targets = vec![WindowPreviewTarget {
            hwnd: WindowIdentity::from_handle(999),
            rect: Rect::default(),
            clip: None,
        }];
        for hwnd in 1..=40 {
            targets.push(WindowPreviewTarget {
                hwnd: WindowIdentity::from_handle(hwnd),
                rect: rect.clone(),
                clip: None,
            });
            targets.push(WindowPreviewTarget {
                hwnd: WindowIdentity::from_handle(hwnd),
                rect: rect.clone(),
                clip: None,
            });
        }
        let selected = bounded_targets(targets);
        assert_eq!(selected.len(), MAX_TARGETS);
        assert_eq!(
            selected
                .iter()
                .map(|target| target.hwnd.hwnd)
                .collect::<Vec<_>>(),
            (1..=16).collect::<Vec<_>>()
        );
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
}
