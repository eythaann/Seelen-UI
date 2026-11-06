//! Opt-in background visuals in the existing Widget HWND, below WebView2 children.

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicU32, Ordering},
};

use seelen_core::state::{NativeBlurOpacityAnimation, NativeBlurRegion, NativeBlurShape};
use windows::{
    Graphics::{IGeometrySource2D, IGeometrySource2D_Impl},
    UI::Composition::{
        CompositionEffectBrush, CompositionPath, CompositionPathGeometry, Compositor,
        ContainerVisual, Desktop::DesktopWindowTarget, SpriteVisual,
    },
    Win32::{
        Foundation::HWND,
        Graphics::Direct2D::{
            Common::{D2D_SIZE_F, D2D1_FIGURE_BEGIN_FILLED, D2D1_FIGURE_END_CLOSED},
            D2D1_ARC_SEGMENT, D2D1_ARC_SIZE_SMALL, D2D1_FACTORY_TYPE_SINGLE_THREADED,
            D2D1_SWEEP_DIRECTION_CLOCKWISE, D2D1CreateFactory, ID2D1Factory, ID2D1Geometry,
        },
        System::WinRT::{
            Composition::ICompositorDesktopInterop,
            Graphics::Direct2D::{IGeometrySource2DInterop, IGeometrySource2DInterop_Impl},
        },
        UI::WindowsAndMessaging::IsWindow,
    },
    core::{Interface, Ref, implement},
};
use windows_numerics::{Vector2, Vector3};

use super::native_blur_effect::{create_blur_brush, ensure_dispatcher};
use crate::{app::get_app_handle, error::Result};

#[path = "native_blur_visibility.rs"]
mod visibility;

const MAX_REGIONS: usize = 32;
const MAX_CLIPS: usize = 16;
static NEXT_SESSION: AtomicU32 = AtomicU32::new(1);

thread_local! {
    static WINDOWS: RefCell<HashMap<String, WindowState>> = RefCell::new(HashMap::new());
}

struct WindowState {
    hwnd: isize,
    session: u32,
    sequence: u32,
    visibility: visibility::Visibility,
    host: Option<NativeBlurHost>,
}

struct NativeBlurHost {
    compositor: Compositor,
    target: DesktopWindowTarget,
    factory: ID2D1Factory,
    root: ContainerVisual,
    regions: HashMap<u32, RegionVisual>,
    order: Vec<u32>,
}

impl NativeBlurHost {
    fn arm_hide(&self, remaining: std::time::Duration) -> Result<()> {
        let property = windows::core::h!("Opacity");
        self.root.StopAnimation(property)?;
        if remaining.is_zero() {
            self.root.SetOpacity(0.0)?;
            return Ok(());
        }
        // A visibility gate, not a second fade: preserve CSS/Sprite opacity until the deadline.
        // Composition executes this even when the WebView JS thread is stalled.
        self.root.SetOpacity(1.0)?;
        let gate = self.compositor.CreateScalarKeyFrameAnimation()?;
        gate.SetDuration(windows::Foundation::TimeSpan {
            Duration: (remaining.as_nanos() / 100).max(1).min(i64::MAX as u128) as i64,
        })?;
        gate.InsertKeyFrame(0.0, 1.0)?;
        gate.InsertKeyFrame(0.999999, 1.0)?;
        gate.InsertKeyFrame(1.0, 0.0)?;
        self.root.StartAnimation(property, &gate)?;
        Ok(())
    }

    fn new(hwnd: isize) -> Result<Self> {
        ensure_dispatcher()?;
        let compositor = Compositor::new()?;
        let interop: ICompositorDesktopInterop = compositor.cast()?;
        // false is essential: HTML text and input remain above the background visuals.
        let target = unsafe { interop.CreateDesktopWindowTarget(HWND(hwnd as *mut _), false)? };
        let root = compositor.CreateContainerVisual()?;
        root.SetRelativeSizeAdjustment(Vector2 { X: 1.0, Y: 1.0 })?;
        target.SetRoot(&root)?;
        let factory = unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)? };
        Ok(Self {
            compositor,
            target,
            factory,
            root,
            regions: HashMap::new(),
            order: Vec::new(),
        })
    }

    fn update(&mut self, regions: Vec<NativeBlurRegion>, scale: f32) -> Result<()> {
        let mut order = Vec::with_capacity(regions.len());
        for mut region in regions {
            region.amount = (region.amount * scale).min(250.0);
            for clip in &mut region.clips {
                clip.x *= scale;
                clip.y *= scale;
                clip.width *= scale;
                clip.height *= scale;
                for radius in &mut clip.radii {
                    radius[0] *= scale;
                    radius[1] *= scale;
                }
            }
            let id = region.id;
            if let Some(visual) = self.regions.get_mut(&id) {
                visual.update(&self.compositor, &self.factory, region)?;
            } else {
                self.regions.insert(
                    id,
                    RegionVisual::new(&self.compositor, &self.factory, region)?,
                );
            }
            order.push(id);
        }
        if self.order != order {
            self.root.Children()?.RemoveAll()?;
            for id in &order {
                self.root.Children()?.InsertAtTop(&self.regions[id].root)?;
            }
            self.regions.retain(|id, _| order.contains(id));
            self.order = order;
        }
        Ok(())
    }
}

impl Drop for NativeBlurHost {
    fn drop(&mut self) {
        let _ = self
            .target
            .SetRoot(None::<&windows::UI::Composition::Visual>);
        let _ = self.target.Close();
        log::debug!("Native blur Composition host released");
    }
}

struct RegionVisual {
    root: ContainerVisual,
    sprite: SpriteVisual,
    _brush: CompositionEffectBrush,
    clips: Vec<CompositionPathGeometry>,
    previous: NativeBlurRegion,
}

impl RegionVisual {
    fn new(
        compositor: &Compositor,
        factory: &ID2D1Factory,
        region: NativeBlurRegion,
    ) -> Result<Self> {
        let root = compositor.CreateContainerVisual()?;
        root.SetRelativeSizeAdjustment(Vector2 { X: 1.0, Y: 1.0 })?;
        let sprite = compositor.CreateSpriteVisual()?;
        let brush = create_blur_brush(compositor, region.amount)?;
        sprite.SetBrush(&brush)?;
        let mut visual = Self {
            root,
            sprite,
            _brush: brush,
            clips: Vec::new(),
            previous: region.clone(),
        };
        visual.rebuild_clips(compositor, region.clips.len())?;
        visual.apply_geometry(factory, &region, true)?;
        visual.apply_opacity(compositor, &region)?;
        #[cfg(test)]
        PERF_COUNTS.with_borrow_mut(|counts| counts.region_created += 1);
        Ok(visual)
    }

    fn rebuild_clips(&mut self, compositor: &Compositor, count: usize) -> Result<()> {
        if !self.clips.is_empty() {
            // Detach the reusable sprite before releasing its old ancestor containers.
            // Removing the root first can close the orphaned subtree on this runtime.
            self.sprite.Parent()?.Children()?.Remove(&self.sprite)?;
        }
        self.root.Children()?.RemoveAll()?;
        self.clips.clear();
        let mut parent = self.root.clone();
        for _ in 0..count {
            let child = compositor.CreateContainerVisual()?;
            child.SetRelativeSizeAdjustment(Vector2 { X: 1.0, Y: 1.0 })?;
            let geometry = compositor.CreatePathGeometry()?;
            child.SetClip(&compositor.CreateGeometricClipWithGeometry(&geometry)?)?;
            parent.Children()?.InsertAtTop(&child)?;
            parent = child;
            self.clips.push(geometry);
        }
        parent.Children()?.InsertAtTop(&self.sprite)?;
        Ok(())
    }

    fn apply_geometry(
        &self,
        factory: &ID2D1Factory,
        region: &NativeBlurRegion,
        rebuild: bool,
    ) -> Result<()> {
        let bounds = &region.clips[0];
        if rebuild || self.previous.clips.first() != region.clips.first() {
            self.sprite.SetOffset(Vector3 {
                X: bounds.x,
                Y: bounds.y,
                Z: 0.0,
            })?;
            self.sprite.SetSize(Vector2 {
                X: bounds.width,
                Y: bounds.height,
            })?;
        }
        for (index, (geometry, shape)) in self.clips.iter().zip(&region.clips).enumerate() {
            if !rebuild && self.previous.clips.get(index) == Some(shape) {
                continue;
            }
            #[cfg(test)]
            PERF_COUNTS.with_borrow_mut(|counts| counts.paths += 1);
            let source: IGeometrySource2D = GeometrySource {
                geometry: create_geometry(factory, shape)?,
                shape: shape.clone(),
            }
            .into();
            geometry.SetPath(&CompositionPath::Create(&source)?)?;
        }
        Ok(())
    }

    fn update(
        &mut self,
        compositor: &Compositor,
        factory: &ID2D1Factory,
        region: NativeBlurRegion,
    ) -> Result<()> {
        if self.previous.amount != region.amount {
            self._brush = create_blur_brush(compositor, region.amount)?;
            self.sprite.SetBrush(&self._brush)?;
        }
        if self.previous.clips != region.clips {
            let rebuild = self.clips.len() != region.clips.len();
            if rebuild {
                self.rebuild_clips(compositor, region.clips.len())?;
            }
            self.apply_geometry(factory, &region, rebuild)?;
        }
        if self.previous.opacity_animation != region.opacity_animation
            || (region.opacity_animation.is_none() && self.previous.opacity != region.opacity)
        {
            self.apply_opacity(compositor, &region)?;
        }
        self.previous = region;
        Ok(())
    }

    fn apply_opacity(&self, compositor: &Compositor, region: &NativeBlurRegion) -> Result<()> {
        let property = windows::core::h!("Opacity");
        self.sprite.StopAnimation(property)?;
        let Some(plan) = &region.opacity_animation else {
            self.sprite.SetOpacity(region.opacity)?;
            return Ok(());
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
            * 1000.0;
        // IPC is asynchronous: skip elapsed curve time rather than replaying the fade.
        let elapsed = ((now - plan.started_at_ms).max(0.0) as f32).min(plan.duration_ms);
        if elapsed >= plan.duration_ms {
            self.sprite.SetOpacity(*plan.values.last().unwrap())?;
            return Ok(());
        }
        let animation = compositor.CreateScalarKeyFrameAnimation()?;
        let linear = compositor.CreateLinearEasingFunction()?;
        animation.SetDuration(windows::Foundation::TimeSpan {
            Duration: ((plan.duration_ms - elapsed) as f64 * 10_000.0).max(1.0) as i64,
        })?;
        let start = opacity_at(plan, elapsed);
        self.sprite.SetOpacity(start)?;
        animation.InsertKeyFrameWithEasingFunction(0.0, start, &linear)?;
        for (i, value) in plan.values.iter().enumerate().skip(1) {
            let time = i as f32 * plan.duration_ms / (plan.values.len() - 1) as f32;
            if time > elapsed {
                animation.InsertKeyFrameWithEasingFunction(
                    ((time - elapsed) / (plan.duration_ms - elapsed)).min(1.0),
                    *value,
                    &linear,
                )?;
            }
        }
        self.sprite.StartAnimation(property, &animation)?;
        Ok(())
    }
}

#[cfg(test)]
impl Drop for RegionVisual {
    fn drop(&mut self) {
        PERF_COUNTS.with_borrow_mut(|counts| counts.region_dropped += 1);
    }
}

fn opacity_at(plan: &NativeBlurOpacityAnimation, elapsed: f32) -> f32 {
    let position = (elapsed / plan.duration_ms).clamp(0.0, 1.0) * (plan.values.len() - 1) as f32;
    let index = (position.floor() as usize).min(plan.values.len() - 2);
    plan.values[index] + (plan.values[index + 1] - plan.values[index]) * (position - index as f32)
}

#[implement(IGeometrySource2D, IGeometrySource2DInterop)]
struct GeometrySource {
    geometry: ID2D1Geometry,
    shape: NativeBlurShape,
}

impl IGeometrySource2D_Impl for GeometrySource_Impl {}

impl IGeometrySource2DInterop_Impl for GeometrySource_Impl {
    fn GetGeometry(&self) -> windows::core::Result<ID2D1Geometry> {
        Ok(self.geometry.clone())
    }

    fn TryGetGeometryUsingFactory(
        &self,
        factory: Ref<'_, ID2D1Factory>,
    ) -> windows::core::Result<ID2D1Geometry> {
        let factory = factory.as_ref().ok_or_else(|| {
            windows::core::Error::from_hresult(windows::core::HRESULT(0x80070057u32 as i32))
        })?;
        create_geometry(factory, &self.shape)
    }
}

fn create_geometry(
    factory: &ID2D1Factory,
    shape: &NativeBlurShape,
) -> windows::core::Result<ID2D1Geometry> {
    let path = unsafe { factory.CreatePathGeometry()? };
    let sink = unsafe { path.Open()? };
    let [tl, tr, br, bl] = shape.radii;
    let (x, y, r, b) = (
        shape.x,
        shape.y,
        shape.x + shape.width,
        shape.y + shape.height,
    );
    unsafe {
        sink.BeginFigure(Vector2 { X: x + tl[0], Y: y }, D2D1_FIGURE_BEGIN_FILLED);
        for (line, end, radius) in [
            ([r - tr[0], y], [r, y + tr[1]], tr),
            ([r, b - br[1]], [r - br[0], b], br),
            ([x + bl[0], b], [x, b - bl[1]], bl),
            ([x, y + tl[1]], [x + tl[0], y], tl),
        ] {
            sink.AddLine(Vector2 {
                X: line[0],
                Y: line[1],
            });
            let point = Vector2 {
                X: end[0],
                Y: end[1],
            };
            if radius[0] > 0.0 && radius[1] > 0.0 {
                sink.AddArc(&D2D1_ARC_SEGMENT {
                    point,
                    size: D2D_SIZE_F {
                        width: radius[0],
                        height: radius[1],
                    },
                    rotationAngle: 0.0,
                    sweepDirection: D2D1_SWEEP_DIRECTION_CLOCKWISE,
                    arcSize: D2D1_ARC_SIZE_SMALL,
                });
            } else {
                sink.AddLine(point);
            }
        }
        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
        sink.Close()?;
    }
    path.cast()
}

fn validate_regions(regions: &[NativeBlurRegion], scale: f32) -> Result<()> {
    if !scale.is_finite() || !(0.25..=8.0).contains(&scale) || regions.len() > MAX_REGIONS {
        return Err("Invalid native blur scale or region count".into());
    }
    let mut ids = HashSet::new();
    let mut area = 0.0;
    for region in regions {
        if !ids.insert(region.id)
            || region.clips.is_empty()
            || region.clips.len() > MAX_CLIPS
            || !region.amount.is_finite()
            || !(0.0..=64.0).contains(&region.amount)
            || !region.opacity.is_finite()
            || !(0.0..=1.0).contains(&region.opacity)
        {
            return Err("Invalid native blur region".into());
        }
        if let Some(plan) = &region.opacity_animation
            && (!plan.started_at_ms.is_finite()
                || !plan.duration_ms.is_finite()
                || !(0.0..=2000.0).contains(&plan.duration_ms)
                || plan.duration_ms == 0.0
                || !(2..=128).contains(&plan.values.len())
                || plan
                    .values
                    .iter()
                    .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value)))
        {
            return Err("Invalid native blur opacity animation".into());
        }
        for clip in &region.clips {
            if ![clip.x, clip.y, clip.width, clip.height]
                .iter()
                .all(|v| v.is_finite() && v.abs() <= 32768.0)
                || clip.width <= 0.0
                || clip.height <= 0.0
                || clip
                    .radii
                    .iter()
                    .flatten()
                    .any(|v| !v.is_finite() || *v < 0.0 || *v > 32768.0)
            {
                return Err("Invalid native blur clipping geometry".into());
            }
        }
        area += region.clips[0].width * region.clips[0].height * scale * scale;
    }
    if area > 64_000_000.0 {
        return Err("Native blur regions exceed the per-widget pixel budget".into());
    }
    Ok(())
}

pub fn reset_window(window: &tauri::WebviewWindow) {
    if let Ok(hwnd) = window.hwnd() {
        reset(window.label().to_string(), hwnd.0 as isize);
    }
}

fn reset(label: String, hwnd: isize) {
    let _ = get_app_handle().run_on_main_thread(move || {
        WINDOWS.with_borrow_mut(|windows| {
            if windows.get(&label).is_some_and(|state| state.hwnd == hwnd) {
                windows.remove(&label);
            }
        });
    });
}

pub fn register_window(window: &tauri::WebviewWindow) -> Result<()> {
    let hwnd = window.hwnd()?.0 as isize;
    let label = window.label().to_string();
    window.on_window_event(move |event| {
        if matches!(event, tauri::WindowEvent::Destroyed) {
            reset(label.clone(), hwnd);
        }
    });
    Ok(())
}

impl crate::tauri_handlers::Handlers {
    pub async fn begin_native_blur_session(webview: tauri::WebviewWindow) -> Result<u32> {
        let hwnd = webview.hwnd()?.0 as isize;
        let label = webview.label().to_string();
        let (tx, rx) = tokio::sync::oneshot::channel();
        get_app_handle().run_on_main_thread(move || {
            let session = NEXT_SESSION.fetch_add(1, Ordering::Relaxed);
            WINDOWS.with_borrow_mut(|windows| {
                windows.insert(
                    label,
                    WindowState {
                        hwnd,
                        session,
                        sequence: 0,
                        visibility: visibility::Visibility::default(),
                        host: None,
                    },
                );
            });
            let _ = tx.send(session);
        })?;
        rx.await
            .map_err(|_| "Native blur UI thread unavailable".into())
    }

    pub async fn sync_native_blur_regions(
        webview: tauri::WebviewWindow,
        session: u32,
        sequence: u32,
        scale: f32,
        regions: Vec<NativeBlurRegion>,
        visibility_epoch: Option<u32>,
    ) -> Result<()> {
        let hwnd = webview.hwnd()?.0 as isize;
        let label = webview.label().to_string();
        let (tx, rx) = tokio::sync::oneshot::channel();
        get_app_handle().run_on_main_thread(move || {
            let result = WINDOWS.with_borrow_mut(|windows| -> Result<()> {
                let state = windows
                    .get_mut(&label)
                    .filter(|state| state.hwnd == hwnd && state.session == session)
                    .ok_or("Native blur document session expired")?;
                if sequence <= state.sequence {
                    return Ok(());
                }
                state.sequence = sequence;
                let epoch = visibility_epoch.unwrap_or(0);
                let newer = epoch > state.visibility.epoch;
                let was_closing = state.visibility.deadline.is_some();
                if !state
                    .visibility
                    .accept_frame(epoch, std::time::Instant::now())
                {
                    if state
                        .visibility
                        .expired(state.visibility.epoch, std::time::Instant::now())
                    {
                        state.host = None;
                    }
                    return Ok(());
                }
                if newer && was_closing {
                    state.host = None;
                }
                let result = (|| -> Result<()> {
                    validate_regions(&regions, scale)?;
                    if regions.is_empty() {
                        state.host = None;
                        return Ok(());
                    }
                    if !unsafe { IsWindow(Some(HWND(hwnd as *mut _))) }.as_bool() {
                        return Err("Native blur Widget window no longer exists".into());
                    }
                    if state.host.is_none() {
                        // Closing never creates a new native surface, even if a late frame arrives.
                        if state.visibility.deadline.is_some() {
                            return Ok(());
                        }
                        state.host = Some(NativeBlurHost::new(hwnd)?);
                        log::info!("Native blur attached below WebView2: {label}");
                    }
                    state.host.as_mut().unwrap().update(regions, scale)
                })();
                if let Err(error) = &result {
                    state.host = None;
                    log::warn!("Native blur disabled for {label}: {error:?}");
                }
                result
            });
            let _ = tx.send(result);
        })?;
        rx.await
            .map_err(|_| crate::error::AppError::from("Native blur UI thread unavailable"))?
    }

    pub async fn set_native_blur_visibility(
        webview: tauri::WebviewWindow,
        session: u32,
        epoch: u32,
        visible: bool,
        deadline_ms: f64,
    ) -> Result<()> {
        if !deadline_ms.is_finite() {
            return Err("Invalid native blur visibility deadline".into());
        }
        let hwnd = webview.hwnd()?.0 as isize;
        let label = webview.label().to_string();
        let (tx, rx) = tokio::sync::oneshot::channel();
        get_app_handle().run_on_main_thread(move || {
            let result = WINDOWS.with_borrow_mut(|windows| -> Result<()> {
                let state = windows.get_mut(&label).filter(|state| state.hwnd == hwnd && state.session == session)
                    .ok_or("Native blur document session expired")?;
                if visible {
                    let was_closing = state.visibility.deadline.is_some();
                    if state.visibility.open(epoch) && was_closing { state.host = None; }
                    return Ok(());
                }
                let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default().as_secs_f64() * 1000.0;
                let remaining = std::time::Duration::from_secs_f64((deadline_ms - now_ms).clamp(0.0, 2000.0) / 1000.0);
                let now = std::time::Instant::now();
                let Some(deadline) = state.visibility.close(epoch, now + remaining) else { return Ok(()); };
                let remaining = deadline.saturating_duration_since(now);
                if remaining.is_zero() {
                    state.host = None;
                } else if let Some(host) = &state.host
                    && let Err(error) = host.arm_hide(remaining) {
                    state.host = None;
                    return Err(error);
                }
                log::trace!("Native blur close: {label} session={session} epoch={epoch} remaining_ms={} late_ms={:.1}",
                    remaining.as_millis(), (now_ms - deadline_ms).max(0.0));
                if !remaining.is_zero() && state.host.is_some() {
                    crate::get_tokio_handle().spawn(async move {
                        tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
                        let _ = get_app_handle().run_on_main_thread(move || {
                            WINDOWS.with_borrow_mut(|windows| {
                                if let Some(state) = windows.get_mut(&label).filter(|state|
                                    state.hwnd == hwnd && state.session == session &&
                                    state.visibility.expired(epoch, std::time::Instant::now())) {
                                    state.host = None;
                                    log::trace!("Native blur deadline cleared: {label} session={session} epoch={epoch}");
                                }
                            });
                        });
                    });
                }
                Ok(())
            });
            let _ = tx.send(result);
        })?;
        rx.await
            .map_err(|_| crate::error::AppError::from("Native blur UI thread unavailable"))?
    }
}

#[cfg(test)]
#[derive(Default, Clone, Copy, Debug)]
struct PerfCounts {
    region_created: usize,
    region_dropped: usize,
    paths: usize,
}

#[cfg(test)]
thread_local! { static PERF_COUNTS: RefCell<PerfCounts> = RefCell::new(PerfCounts::default()); }

#[cfg(test)]
#[path = "native_blur_performance.rs"]
mod performance;

#[cfg(test)]
mod tests {
    use super::*;

    fn region() -> NativeBlurRegion {
        NativeBlurRegion {
            id: 1,
            amount: 18.0,
            opacity: 1.0,
            opacity_animation: None,
            clips: vec![NativeBlurShape {
                x: 10.0,
                y: 20.0,
                width: 320.0,
                height: 200.0,
                radii: [[22.0, 22.0]; 4],
            }],
        }
    }

    #[test]
    fn creates_composition_effect_and_elliptical_clip_resources() {
        ensure_dispatcher().unwrap();
        let compositor = Compositor::new().unwrap();
        let factory: ID2D1Factory =
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).unwrap() };
        let mut model = region();
        model.clips[0].radii = [[12.0, 22.0], [30.0, 14.0], [20.0, 20.0], [0.0, 0.0]];
        let mut visual = RegionVisual::new(&compositor, &factory, model.clone()).unwrap();
        assert_eq!(visual.sprite.Size().unwrap().X, 320.0);
        model.opacity_animation = Some(NativeBlurOpacityAnimation {
            id: 1,
            started_at_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64()
                * 1000.0,
            duration_ms: 100.0,
            values: vec![1.0, 0.5, 0.0],
        });
        visual.update(&compositor, &factory, model.clone()).unwrap();
        model.opacity_animation = None;
        model.clips[0].width = 400.0;
        model.opacity = 0.4;
        visual.update(&compositor, &factory, model).unwrap();
        assert_eq!(visual.sprite.Size().unwrap().X, 400.0);
        assert!((visual.sprite.Opacity().unwrap() - 0.4).abs() < 0.001);
    }

    #[test]
    fn rejects_invalid_geometry_and_duplicate_ids() {
        assert!(validate_regions(&[region()], 1.5).is_ok());
        assert!(validate_regions(&[region(), region()], 1.0).is_err());
        let mut invalid = region();
        invalid.clips[0].radii[0][0] = f32::NAN;
        assert!(validate_regions(&[invalid], 1.0).is_err());
    }

    #[test]
    fn bounds_gpu_work_and_accepts_empty_cleanup() {
        assert!(validate_regions(&[], 1.0).is_ok());
        assert!(validate_regions(&[region()], f32::INFINITY).is_err());
        let mut large = region();
        large.clips[0].width = 16000.0;
        large.clips[0].height = 16000.0;
        assert!(validate_regions(&[large], 1.0).is_err());
    }

    #[test]
    fn opacity_timeline_skips_elapsed_ipc_time_and_validates_input() {
        let plan = NativeBlurOpacityAnimation {
            id: 1,
            started_at_ms: 0.0,
            duration_ms: 100.0,
            values: vec![1.0, 0.5, 0.0],
        };
        assert!((opacity_at(&plan, 75.0) - 0.25).abs() < 0.001);
        assert_eq!(opacity_at(&plan, 100.0), 0.0);
        let mut model = region();
        model.opacity_animation = Some(plan);
        assert!(validate_regions(&[model.clone()], 1.0).is_ok());
        model.opacity_animation.as_mut().unwrap().values = vec![1.0];
        assert!(validate_regions(&[model], 1.0).is_err());
    }
}
