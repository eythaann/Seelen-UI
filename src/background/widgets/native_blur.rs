//! Opt-in background visuals in the existing Widget HWND, below WebView2 children.

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
};

use seelen_core::state::{
    NativeBlurCloseAnimation, NativeBlurOpacityAnimation, NativeBlurRegion, NativeBlurShape,
};
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
        UI::WindowsAndMessaging::{
            GW_OWNER, GetForegroundWindow, GetWindow, IsWindow, IsWindowVisible,
        },
    },
    core::{Interface, Ref, implement},
};
use windows_numerics::{Vector2, Vector3};

use super::native_blur_effect::{create_blur_brush, ensure_dispatcher};
use crate::{app::get_app_handle, error::Result};

#[path = "native_blur_focus.rs"]
mod focus;
#[path = "native_blur_visibility.rs"]
mod visibility;

const MAX_REGIONS: usize = 32;
const MAX_CLIPS: usize = 16;
static NEXT_SESSION: AtomicU32 = AtomicU32::new(1);
static FOCUS_GENERATION: AtomicU64 = AtomicU64::new(0);

thread_local! {
    static WINDOWS: RefCell<HashMap<String, WindowState>> = RefCell::new(HashMap::new());
    static SWITCH_CLOSE: RefCell<Option<SwitchClose>> = const { RefCell::new(None) };
}

struct SwitchClose {
    label: String,
    session: u32,
    started: tokio::sync::oneshot::Sender<()>,
}

struct WindowState {
    hwnd: isize,
    session: u32,
    sequence: u32,
    visibility: visibility::Visibility,
    host: Option<NativeBlurHost>,
    focus: focus::PopupFocus,
    animate_close: bool,
}

struct NativeBlurHost {
    compositor: Compositor,
    target: DesktopWindowTarget,
    factory: ID2D1Factory,
    root: ContainerVisual,
    regions: HashMap<u32, RegionVisual>,
    order: Vec<u32>,
    close_path_armed: bool,
    motion_id: Option<u32>,
}

impl NativeBlurHost {
    fn release_immediately(slot: &mut Option<Self>) -> Result<()> {
        if let Some(host) = slot.take() {
            let result = host.hide_immediately();
            drop(host); // Always detach/close, including StopAnimation/SetOpacity failure.
            return result;
        }
        Ok(())
    }

    fn hide_immediately(&self) -> Result<()> {
        self.root.StopAnimation(windows::core::h!("Opacity"))?;
        self.root.SetOpacity(0.0)?;
        Ok(())
    }
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
            close_path_armed: false,
            motion_id: None,
        })
    }

    fn arm_region_path(&self, plan: &NativeBlurCloseAnimation) -> Result<()> {
        let region = self
            .regions
            .get(&plan.region_id)
            .filter(|_| self.regions.len() == 1)
            .ok_or("Native blur path requires one matching region")?;
        let result = region.arm_close_path(&self.compositor, &self.factory, plan);
        if result.is_err() {
            let _ = region.clips[0].StopAnimation(windows::core::h!("Path"));
            let _ = region.apply_geometry(&self.factory, &region.previous, true);
            let _ = region.apply_opacity(&self.compositor, &region.previous);
        }
        result
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
                if let Some(points) = &mut clip.polygon {
                    for point in points {
                        point[0] *= scale;
                        point[1] *= scale;
                    }
                }
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
        // Always detach and close, even if one of them fails.
        let detached = self
            .target
            .SetRoot(None::<&windows::UI::Composition::Visual>);
        let closed = self.target.Close();
        if let Err(error) = detached.and(closed) {
            log::warn!("Native blur Composition host release failed: {error}");
        }
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
    fn arm_close_path(
        &self,
        compositor: &Compositor,
        factory: &ID2D1Factory,
        plan: &NativeBlurCloseAnimation,
    ) -> Result<()> {
        let count = plan.outlines.len();
        let opacity = &plan.opacity;
        if !(2..=32).contains(&count)
            || plan.outlines.iter().any(|points| {
                !(3..=64).contains(&points.len()) || points.len() != plan.outlines[0].len()
            })
        {
            return Err("Invalid native blur close outline count".into());
        }
        let mut region = self.previous.clone();
        region.amount = 1.0;
        region.opacity_animation = Some(opacity.clone());
        let mut union = [
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ];
        let mut shapes = Vec::with_capacity(count);
        for points in &plan.outlines {
            let mut bounds = [
                f32::INFINITY,
                f32::INFINITY,
                f32::NEG_INFINITY,
                f32::NEG_INFINITY,
            ];
            for [x, y] in points {
                bounds = [
                    bounds[0].min(*x),
                    bounds[1].min(*y),
                    bounds[2].max(*x),
                    bounds[3].max(*y),
                ];
            }
            union = [
                union[0].min(bounds[0]),
                union[1].min(bounds[1]),
                union[2].max(bounds[2]),
                union[3].max(bounds[3]),
            ];
            let shape = NativeBlurShape {
                x: bounds[0],
                y: bounds[1],
                width: bounds[2] - bounds[0],
                height: bounds[3] - bounds[1],
                radii: [[0.0; 2]; 4],
                polygon: Some(points.clone()),
            };
            region.clips = vec![shape.clone()];
            validate_regions(std::slice::from_ref(&region), plan.scale)?;
            shapes.push(shape);
        }
        let [left, top, right, bottom] = union;
        if (right - left) * (bottom - top) * plan.scale * plan.scale > 64_000_000.0 {
            return Err("Native blur close outline exceeds the pixel budget".into());
        }
        let path = |mut shape: NativeBlurShape| -> Result<CompositionPath> {
            for point in shape.polygon.as_mut().unwrap() {
                point[0] *= plan.scale;
                point[1] *= plan.scale;
            }
            let source: IGeometrySource2D = GeometrySource {
                geometry: create_geometry(factory, &shape)?,
                shape,
            }
            .into();
            Ok(CompositionPath::Create(&source)?)
        };
        let paths = shapes
            .iter()
            .cloned()
            .map(path)
            .collect::<Result<Vec<_>>>()?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
            * 1000.0;
        let elapsed = (now - opacity.started_at_ms).max(0.0) as f32;
        let remaining = opacity.duration_ms - elapsed;
        if remaining <= 0.0 {
            return Err("Native blur close animation already elapsed".into());
        }
        let position = elapsed * (count - 1) as f32 / opacity.duration_ms;
        let index = (position as usize).min(count - 2);
        let fraction = position - index as f32;
        let mut initial = shapes[index].clone();
        for (point, next) in initial
            .polygon
            .as_mut()
            .unwrap()
            .iter_mut()
            .zip(&plan.outlines[index + 1])
        {
            for axis in 0..2 {
                point[axis] += (next[axis] - point[axis]) * fraction;
            }
        }
        let animation = compositor.CreatePathKeyFrameAnimation()?;
        let linear = compositor.CreateLinearEasingFunction()?;
        animation.SetDuration(windows::Foundation::TimeSpan {
            Duration: (remaining as f64 * 10_000.0).max(1.0) as i64,
        })?;
        animation.InsertKeyFrameWithEasingFunction(0.0, &path(initial)?, &linear)?;
        for (i, frame) in paths.iter().enumerate().skip(index + 1) {
            let progress = ((i as f32 * opacity.duration_ms / (count - 1) as f32 - elapsed)
                / remaining)
                .clamp(0.0, 1.0);
            animation.InsertKeyFrameWithEasingFunction(progress, frame, &linear)?;
        }
        self.sprite.SetOffset(Vector3 {
            X: left * plan.scale,
            Y: top * plan.scale,
            Z: 0.0,
        })?;
        self.sprite.SetSize(Vector2 {
            X: (right - left) * plan.scale,
            Y: (bottom - top) * plan.scale,
        })?;
        self.clips[0].StartAnimation(windows::core::h!("Path"), &animation)?;
        self.apply_opacity(compositor, &region)
    }

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
    if let Some(points) = &shape.polygon
        && let Some(first) = points.first()
    {
        unsafe {
            sink.BeginFigure(
                Vector2 {
                    X: first[0],
                    Y: first[1],
                },
                D2D1_FIGURE_BEGIN_FILLED,
            );
            for point in &points[1..] {
                sink.AddLine(Vector2 {
                    X: point[0],
                    Y: point[1],
                });
            }
            sink.EndFigure(D2D1_FIGURE_END_CLOSED);
            sink.Close()?;
        }
        return path.cast();
    }
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
        for (index, clip) in region.clips.iter().enumerate() {
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
            if let Some(points) = &clip.polygon {
                if index != 0
                    || !(3..=64).contains(&points.len())
                    || points.iter().any(|point| {
                        !point.iter().all(|v| v.is_finite() && v.abs() <= 32768.0)
                            || point[0] < clip.x - 0.05
                            || point[0] > clip.x + clip.width + 0.05
                            || point[1] < clip.y - 0.05
                            || point[1] > clip.y + clip.height + 0.05
                    })
                {
                    return Err("Invalid native blur projected outline".into());
                }
                let mut area = 0.0;
                for i in 0..points.len() {
                    let (a, b, c) = (
                        points[i],
                        points[(i + 1) % points.len()],
                        points[(i + 2) % points.len()],
                    );
                    area += a[0] * b[1] - b[0] * a[1];
                    if (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0]) < -0.01 {
                        return Err("Native blur outline must be convex and front-facing".into());
                    }
                }
                if area < 0.001 {
                    return Err("Native blur projected outline is degenerate".into());
                }
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

pub(super) fn on_focus_changed(focused: isize, owner: isize) {
    let generation = FOCUS_GENERATION.fetch_add(1, Ordering::AcqRel) + 1;
    let _ = get_app_handle().run_on_main_thread(move || {
        // Do not let a queued loss hide a popup that already regained focus.
        if generation != FOCUS_GENERATION.load(Ordering::Acquire)
            || unsafe { GetForegroundWindow() }.0 as isize != focused
        {
            return;
        }
        apply_popup_focus(focused, owner);
    });
}

fn apply_popup_focus(focused: isize, owner: isize) {
    WINDOWS.with_borrow_mut(|windows| {
        for (label, state) in windows.iter_mut() {
            let Ok(parsed) = super::webview::WidgetWebviewLabel::try_from_raw(label) else {
                continue;
            };
            // Built-in popup lifecycle only; third-party widgets may opt out of focus-loss hide.
            if !parsed.widget_id.to_string().starts_with("@seelen/") {
                continue;
            }
            let popup = crate::resources::RESOURCES
                .widgets
                .read_sync(&parsed.widget_id, |_, widget| {
                    widget.preset == seelen_core::state::WidgetPreset::Popup
                })
                .unwrap_or(false);
            if !popup {
                continue;
            }
            let Some(restore) = state.focus.observe(state.hwnd, focused, owner) else {
                continue;
            };
            if state.animate_close {
                continue;
            }
            if restore && !state.visibility.may_restore_focus_visual() {
                continue;
            }
            if let Some(host) = state.host.as_mut() {
                let result = if restore {
                    host.root.SetOpacity(1.0).map_err(Into::into)
                } else {
                    host.hide_immediately()
                };
                if let Err(error) = result {
                    log::warn!("Popup focus blur update failed for {label}: {error}");
                    let _ = NativeBlurHost::release_immediately(&mut state.host);
                }
            }
        }
    });
}

/// Called only on Tauri's UI thread, before starting the next toolbar popup.
/// No Widget-manager guard is held, and cleanup failure never prevents opening.
pub(super) fn prepare_popup_switch(target: &seelen_core::resource::WidgetId) {
    use seelen_core::state::WidgetPreset;
    WINDOWS.with_borrow_mut(|windows| {
        for (label, state) in windows.iter_mut() {
            let Ok(parsed) = super::webview::WidgetWebviewLabel::try_from_raw(label) else {
                continue;
            };
            if &parsed.widget_id == target {
                state.visibility.allow_popup_trigger();
                continue;
            }
            let popup = crate::resources::RESOURCES
                .widgets
                .read_sync(&parsed.widget_id, |_, widget| {
                    widget.preset == WidgetPreset::Popup && widget.native_blur_switch != Some(true)
                })
                .unwrap_or(false);
            // Animated popups keep their blur until the existing focus-loss hide runs.
            if !popup || state.animate_close {
                continue;
            }
            state
                .visibility
                .suppress_for_popup_switch(std::time::Instant::now());
            if let Err(error) = NativeBlurHost::release_immediately(&mut state.host) {
                log::warn!("Popup switch blur cleanup failed for {label}: {error}");
            }
        }
    });
}

fn supports_switch_close(widget: &seelen_core::state::Widget, animate_close: bool) -> bool {
    widget.native_blur_switch == Some(true)
        || (animate_close
            && widget.preset == seelen_core::state::WidgetPreset::Popup
            && widget.id.to_string().starts_with("@seelen/"))
}

/// Hold cold starts for the visible animated popup or a widget that opted into the same handshake.
pub(super) fn request_switch_close(
    target: &seelen_core::resource::WidgetId,
) -> Option<tokio::sync::oneshot::Receiver<()>> {
    use tauri::Emitter;
    let outgoing = WINDOWS.with_borrow(|windows| {
        let foreground = unsafe { GetForegroundWindow() }.0 as isize;
        let mut candidates = windows.iter().filter_map(|(label, state)| {
            let host = state.host.as_ref()?;
            if !unsafe { IsWindowVisible(HWND(state.hwnd as *mut _)) }.as_bool() {
                return None;
            }
            let parsed = super::webview::WidgetWebviewLabel::try_from_raw(label).ok()?;
            if &parsed.widget_id == target {
                return None;
            }
            let participates = crate::resources::RESOURCES
                .widgets
                .read_sync(&parsed.widget_id, |_, widget| {
                    supports_switch_close(widget, state.animate_close)
                })
                .unwrap_or(false);
            participates.then_some((
                label.clone(),
                state.session,
                state.hwnd == foreground,
                (host.close_path_armed || host.motion_id.is_some())
                    .then(|| host.compositor.clone()),
            ))
        });
        let first = candidates.next()?;
        if first.2 {
            return Some(first);
        }
        match candidates.next() {
            None => Some(first),
            Some(second) if second.2 => Some(second),
            Some(_) => candidates.find(|candidate| candidate.2),
        }
    });
    let (label, session, _, armed_compositor) = outgoing?;
    let (started, wait) = tokio::sync::oneshot::channel();
    if let Some(compositor) = armed_compositor {
        signal_after_commit(&compositor, started);
        return Some(wait);
    }
    SWITCH_CLOSE.with_borrow_mut(|pending| {
        *pending = Some(SwitchClose {
            label: label.clone(),
            session,
            started,
        })
    });
    if get_app_handle()
        .emit_to(
            label,
            seelen_core::handlers::SeelenEvent::WidgetNativeBlurClose,
            (),
        )
        .is_err()
    {
        SWITCH_CLOSE.with_borrow_mut(|pending| *pending = None);
        return None;
    }
    Some(wait)
}

fn take_switch_close(label: &str, session: u32) -> Option<tokio::sync::oneshot::Sender<()>> {
    SWITCH_CLOSE.with_borrow_mut(|pending| {
        if pending
            .as_ref()
            .is_some_and(|close| close.label == label && close.session == session)
        {
            return pending.take().map(|close| close.started);
        }
        None
    })
}

fn signal_switch_close(label: &str, session: u32) {
    if let Some(started) = take_switch_close(label, session) {
        let _ = started.send(());
    }
}

fn signal_after_commit(compositor: &Compositor, started: tokio::sync::oneshot::Sender<()>) {
    if let Ok(commit) = compositor.RequestCommitAsync() {
        crate::get_tokio_handle().spawn(async move {
            if commit.await.is_ok() {
                let _ = started.send(());
            }
        });
    }
}

pub(super) fn allow_popup_trigger(label: String, generation: Option<u64>) {
    let _ = get_app_handle().run_on_main_thread(move || {
        if !super::popup_request_is_current(generation) {
            return;
        }
        WINDOWS.with_borrow_mut(|windows| {
            if let Some(state) = windows.get_mut(&label) {
                state.visibility.allow_popup_trigger();
            }
        });
    });
}

impl crate::tauri_handlers::Handlers {
    pub async fn begin_native_blur_session(webview: tauri::WebviewWindow) -> Result<u32> {
        let hwnd = webview.hwnd()?.0 as isize;
        let label = webview.label().to_string();
        let (tx, rx) = tokio::sync::oneshot::channel();
        get_app_handle().run_on_main_thread(move || {
            let session = NEXT_SESSION.fetch_add(1, Ordering::Relaxed);
            let foreground = unsafe { GetForegroundWindow() };
            let owner = unsafe { GetWindow(foreground, GW_OWNER) }.unwrap_or_default();
            WINDOWS.with_borrow_mut(|windows| {
                windows.insert(
                    label,
                    WindowState {
                        hwnd,
                        session,
                        sequence: 0,
                        visibility: visibility::Visibility::default(),
                        host: None,
                        focus: focus::PopupFocus::new(
                            hwnd,
                            foreground.0 as isize,
                            owner.0 as isize,
                        ),
                        animate_close: false,
                    },
                );
            });
            let _ = tx.send(session);
        })?;
        rx.await
            .map_err(|_| "Native blur UI thread unavailable".into())
    }

    // The signature mirrors the command declaration in `libs/core`.
    #[allow(clippy::too_many_arguments)]
    pub async fn sync_native_blur_regions(
        webview: tauri::WebviewWindow,
        session: u32,
        sequence: u32,
        scale: f32,
        regions: Vec<NativeBlurRegion>,
        visibility_epoch: Option<u32>,
        animate_close: Option<bool>,
        motion: Option<NativeBlurCloseAnimation>,
    ) -> Result<bool> {
        let label = webview.label().to_string();
        let (tx, rx) = tokio::sync::oneshot::channel();
        get_app_handle().run_on_main_thread(move || {
            let result = WINDOWS.with_borrow_mut(|windows| -> Result<bool> {
                // The session already owns the HWND. Resolving it through Tauri again can
                // stall behind another popup's WebView creation.
                let state = windows.get_mut(&label).filter(|state| state.session == session)
                    .ok_or("Native blur document session expired")?;
                let hwnd = state.hwnd;
                if sequence <= state.sequence {
                    return Ok(false);
                }
                state.sequence = sequence;
                let epoch = visibility_epoch.unwrap_or(0);
                let newer = epoch > state.visibility.epoch;
                let was_closing = state.visibility.deadline.is_some();
                if !state.visibility.accept_frame(epoch, std::time::Instant::now()) {
                    if state.visibility.expired(state.visibility.epoch, std::time::Instant::now()) {
                        state.host = None;
                    }
                    return Ok(false);
                }
                if newer && was_closing { state.host = None; }
                if state.host.as_ref().is_some_and(|host| host.close_path_armed) {
                    return Ok(false);
                }
                let result = (|| -> Result<bool> {
                    validate_regions(&regions, scale)?;
                    state.animate_close = animate_close.unwrap_or(false);
                    let motion = motion.as_ref().filter(|plan|
                        !was_closing && regions.len() == 1 && regions[0].id == plan.region_id && plan.scale == scale);
                    if let Some(active) = state.host.as_ref().and_then(|host| host.motion_id) {
                        if !newer && motion.is_some_and(|plan| plan.opacity.id == active) { return Ok(true); }
                        // A different region, animation, or epoch owns subsequent geometry.
                        state.host = None;
                    }
                    if regions.is_empty() {
                        state.host = None;
                        signal_switch_close(&label, session);
                        return Ok(false);
                    }
                    if !unsafe { IsWindow(Some(HWND(hwnd as *mut _))) }.as_bool() {
                        return Err("Native blur Widget window no longer exists".into());
                    }
                    if state.host.is_none() {
                        // Closing never creates a new native surface, even if a late frame arrives.
                        if state.visibility.deadline.is_some() { return Ok(false); }
                        state.host = Some(NativeBlurHost::new(hwnd)?);
                        log::info!("Native blur attached below WebView2: {label}");
                    }
                    let host = state.host.as_mut().unwrap();
                    host.update(regions, scale)?;
                    if let Some(plan) = motion.filter(|_| crate::platform::info::is_windows_11()) {
                        match host.arm_region_path(plan) {
                            Ok(()) => {
                                host.motion_id = Some(plan.opacity.id);
                                let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default().as_secs_f64() * 1000.0;
                                let remaining_ms = (plan.opacity.started_at_ms + f64::from(plan.opacity.duration_ms) - now_ms)
                                    .clamp(0.0, 2000.0);
                                if let Err(error) = host.arm_hide(std::time::Duration::from_secs_f64(remaining_ms / 1000.0)) {
                                    log::debug!("Native blur motion deadline unavailable for {label}: {error:?}");
                                } else if let Some(started) = take_switch_close(&label, session) {
                                    signal_after_commit(&host.compositor, started);
                                }
                            }
                            Err(error) => log::debug!("Native blur motion outline unavailable for {label}: {error:?}"),
                        }
                    }
                    if state.focus.suppressed() && !state.animate_close { host.hide_immediately()?; }
                    Ok(host.motion_id.is_some())
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
        close_animation: Option<NativeBlurCloseAnimation>,
    ) -> Result<bool> {
        if !deadline_ms.is_finite() {
            return Err("Invalid native blur visibility deadline".into());
        }
        let label = webview.label().to_string();
        let (tx, rx) = tokio::sync::oneshot::channel();
        get_app_handle().run_on_main_thread(move || {
            let result = WINDOWS.with_borrow_mut(|windows| -> Result<bool> {
                let state = windows.get_mut(&label).filter(|state| state.session == session)
                    .ok_or("Native blur document session expired")?;
                let hwnd = state.hwnd;
                if visible {
                    let was_closing = state.visibility.deadline.is_some();
                    if state.visibility.open(epoch) && was_closing {
                        state.host = None;
                    }
                    return Ok(false);
                }
                let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default().as_secs_f64() * 1000.0;
                let now = std::time::Instant::now();
                let requested = std::time::Duration::from_secs_f64((deadline_ms - now_ms).clamp(0.0, 2000.0) / 1000.0);
                let remaining = if state.focus.suppressed() && !state.animate_close {
                    std::time::Duration::ZERO
                } else { requested };
                let Some(deadline) = state.visibility.close(epoch, now + remaining) else {
                    return Ok(false);
                };
                let remaining = deadline.saturating_duration_since(now);
                if remaining.is_zero() {
                    // Drop even if hiding fails; never retain a half-closed native background.
                    let result = NativeBlurHost::release_immediately(&mut state.host);
                    signal_switch_close(&label, session);
                    result?;
                } else if let Some(host) = &mut state.host {
                    if let Err(error) = host.arm_hide(remaining) {
                        state.host = None;
                        signal_switch_close(&label, session);
                        return Err(error);
                    }
                    if let Some(plan) = close_animation.as_ref().filter(|_| state.animate_close && !host.close_path_armed && crate::platform::info::is_windows_11()) {
                        match host.arm_region_path(plan) {
                            Ok(()) => {
                                host.close_path_armed = true;
                                if let Some(started) = take_switch_close(&label, session) {
                                    signal_after_commit(&host.compositor, started);
                                }
                            }
                            Err(error) => log::debug!("Native blur close outline unavailable for {label}: {error:?}"),
                        }
                    }
                } else {
                    signal_switch_close(&label, session);
                }
                let path_armed = state.host.as_ref().is_some_and(|host| host.close_path_armed);
                log::trace!("Native blur close: {label} session={session} epoch={epoch} remaining_ms={} late_ms={:.1} animate_close={} focus_suppressed={}",
                    remaining.as_millis(), (now_ms - deadline_ms).max(0.0), state.animate_close, state.focus.suppressed());
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
                Ok(path_armed)
            });
            let _ = tx.send(result);
        })?;
        rx.await
            .map_err(|_| crate::error::AppError::from("Native blur UI thread unavailable"))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn third_party_switch_close_requires_an_explicit_opt_in() {
        use seelen_core::state::Widget;
        let overlay: Widget = serde_json::from_str(
            r#"{"id":"@third-party/overlay","preset":"Overlay","nativeBlurSwitch":true}"#,
        )
        .unwrap();
        let popup: Widget =
            serde_json::from_str(r#"{"id":"@third-party/popup","preset":"Popup"}"#).unwrap();
        let builtin: Widget =
            serde_json::from_str(r#"{"id":"@seelen/popup","preset":"Popup"}"#).unwrap();
        assert!(supports_switch_close(&overlay, false));
        assert!(!supports_switch_close(&popup, true));
        assert!(supports_switch_close(&builtin, true));
        assert!(!supports_switch_close(&builtin, false));
    }

    #[test]
    fn projected_outline_validates_and_builds_composition_clip_without_changing_legacy_shapes() {
        let old: NativeBlurShape = serde_json::from_str(
            r#"{"x":10,"y":20,"width":320,"height":200,"radii":[[0,0],[0,0],[0,0],[0,0]]}"#,
        )
        .unwrap();
        assert!(old.polygon.is_none());
        let mut model = region();
        model.clips[0].polygon = Some(vec![
            [20.0, 20.0],
            [320.0, 20.0],
            [330.0, 220.0],
            [10.0, 220.0],
        ]);
        validate_regions(&[model.clone()], 1.5).unwrap();
        ensure_dispatcher().unwrap();
        let compositor = Compositor::new().unwrap();
        let factory =
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).unwrap() };
        let mut visual = RegionVisual::new(&compositor, &factory, model.clone()).unwrap();
        model.clips[0].polygon.as_mut().unwrap()[0][0] += 2.0;
        visual.update(&compositor, &factory, model.clone()).unwrap();
        model.clips[0].polygon.as_mut().unwrap()[1] = [5.0, 500.0];
        assert!(validate_regions(&[model], 1.0).is_err());
    }

    #[test]
    fn projected_outline_rejects_concave_nonfinite_and_unbounded_point_lists() {
        for points in [
            vec![[20.0, 20.0]; 65],
            vec![[f32::NAN, 20.0]; 4],
            vec![
                [10.0, 20.0],
                [330.0, 20.0],
                [100.0, 100.0],
                [330.0, 220.0],
                [10.0, 220.0],
            ],
        ] {
            let mut model = region();
            model.clips[0].polygon = Some(points);
            assert!(validate_regions(&[model], 1.0).is_err());
        }
    }

    #[test]
    fn projected_close_path_submits_keyframes_to_geometric_clip() {
        if !crate::platform::info::is_windows_11() {
            return;
        }
        ensure_dispatcher().unwrap();
        let compositor = Compositor::new().unwrap();
        let factory =
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).unwrap() };
        let visual = RegionVisual::new(&compositor, &factory, region()).unwrap();
        let mut plan = NativeBlurCloseAnimation {
            region_id: 1,
            scale: 1.0,
            outlines: (0..32)
                .map(|index| {
                    let progress = index as f32 / 31.0;
                    vec![
                        [10.0, 20.0],
                        [330.0, 20.0],
                        [330.0 - 40.0 * progress, 220.0 - 180.0 * progress],
                        [10.0 + 40.0 * progress, 220.0 - 180.0 * progress],
                    ]
                })
                .collect(),
            opacity: NativeBlurOpacityAnimation {
                id: 1,
                started_at_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64()
                    * 1000.0,
                duration_ms: 1000.0,
                values: vec![1.0, 0.0],
            },
        };
        visual.arm_close_path(&compositor, &factory, &plan).unwrap();
        plan.outlines[1].pop();
        assert!(visual.arm_close_path(&compositor, &factory, &plan).is_err());
    }

    fn region() -> NativeBlurRegion {
        NativeBlurRegion {
            id: 1,
            amount: 18.0,
            opacity: 1.0,
            opacity_animation: None,
            clips: vec![NativeBlurShape {
                polygon: None,
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
