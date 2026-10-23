//! Resource/CPU microbenchmark, no visible window. Not GPU timing or a leak proof.
use super::super::native_blur_effect::TEST_BRUSH_CREATIONS;
use super::*;
use std::time::Instant;

fn model(id: u32) -> NativeBlurRegion {
    let shape = NativeBlurShape {
        x: 20.0,
        y: 20.0,
        width: 320.0,
        height: 200.0,
        radii: [[18.0, 18.0]; 4],
    };
    NativeBlurRegion {
        id,
        amount: 18.0,
        opacity: 1.0,
        opacity_animation: None,
        clips: vec![shape.clone(), shape.clone(), shape.clone(), shape],
    }
}

#[test]
#[ignore = "Explicit reproducible resource benchmark; run with --ignored --nocapture --test-threads=1"]
fn resource_baseline() {
    ensure_dispatcher().unwrap();
    for count in [1, 8, 32] {
        PERF_COUNTS.with_borrow_mut(|counts| *counts = PerfCounts::default());
        TEST_BRUSH_CREATIONS.with(|value| value.set(0));
        let compositor = Compositor::new().unwrap();
        let factory: ID2D1Factory =
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).unwrap() };
        let start = Instant::now();
        let mut visuals: Vec<_> = (0..count)
            .map(|id| RegionVisual::new(&compositor, &factory, model(id)).unwrap())
            .collect();
        let create_us = start.elapsed().as_micros();
        PERF_COUNTS.with_borrow_mut(|counts| counts.paths = 0);
        let start = Instant::now();
        for step in 0..40 {
            for (id, visual) in visuals.iter_mut().enumerate() {
                let mut region = model(id as u32);
                region.clips[0].x += step as f32 + 1.0;
                visual.update(&compositor, &factory, region).unwrap();
            }
        }
        let update_us = start.elapsed().as_micros();
        let paths = PERF_COUNTS.with_borrow(|counts| counts.paths);
        let brushes = TEST_BRUSH_CREATIONS.with(|value| value.get());
        assert_eq!(
            brushes, count as usize,
            "Geometry changes must reuse effect brushes"
        );
        drop(visuals);
        let counts = PERF_COUNTS.with_borrow(|counts| *counts);
        assert_eq!(counts.region_created, counts.region_dropped);
        println!(
            "NATIVE_BLUR_PERF {{\"regions\":{count},\"updates\":40,\"clips\":4,\"createUs\":{create_us},\"updateUs\":{update_us},\"paths\":{paths},\"brushes\":{brushes},\"created\":{},\"dropped\":{}}}",
            counts.region_created, counts.region_dropped
        );
    }
}

fn private_bytes() -> usize {
    use windows::Win32::System::{
        ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX},
        Threading::GetCurrentProcess,
    };
    let mut counters = PROCESS_MEMORY_COUNTERS_EX::default();
    unsafe {
        GetProcessMemoryInfo(
            GetCurrentProcess(),
            std::ptr::from_mut(&mut counters).cast(),
            std::mem::size_of_val(&counters) as u32,
        )
        .unwrap();
    }
    counters.PrivateUsage
}

fn drain_dispatcher() {
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage,
    };
    for _ in 0..10 {
        let mut message = MSG::default();
        unsafe {
            while PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[test]
#[ignore = "Hidden HWND lifecycle stress; run explicitly, not a visible GPU benchmark"]
fn lifecycle_stress() {
    use windows::{
        Win32::UI::WindowsAndMessaging::{CreateWindowExW, DestroyWindow, WS_POPUP},
        core::w,
    };
    ensure_dispatcher().unwrap();
    let hwnd = unsafe {
        CreateWindowExW(
            Default::default(),
            w!("STATIC"),
            w!("Seelen native blur resource test"),
            WS_POPUP,
            0,
            0,
            320,
            200,
            None,
            None,
            None,
            None,
        )
        .unwrap()
    };
    struct HiddenWindow(HWND);
    impl Drop for HiddenWindow {
        fn drop(&mut self) {
            let _ = unsafe { DestroyWindow(self.0) };
        }
    }
    let _window = HiddenWindow(hwnd);
    PERF_COUNTS.with_borrow_mut(|counts| *counts = PerfCounts::default());
    let mut samples = Vec::new();
    for cycle in 1..=500 {
        let mut host = NativeBlurHost::new(hwnd.0 as isize).unwrap();
        host.update((0..8).map(model).collect(), 1.0).unwrap();
        assert_eq!(host.regions.len(), 8);
        assert_eq!(host.root.Children().unwrap().Count().unwrap(), 8);
        host.update(Vec::new(), 1.0).unwrap();
        assert!(host.regions.is_empty());
        assert_eq!(host.root.Children().unwrap().Count().unwrap(), 0);
        drop(host);
        if cycle % 20 == 0 {
            drain_dispatcher();
        }
        if [20, 100, 300, 500].contains(&cycle) {
            samples.push(serde_json::json!({ "cycle": cycle, "privateBytes": private_bytes() }));
        }
    }
    let counts = PERF_COUNTS.with_borrow(|counts| *counts);
    assert_eq!(counts.region_created, 4000);
    assert_eq!(counts.region_created, counts.region_dropped);
    println!(
        "NATIVE_BLUR_LIFECYCLE {}",
        serde_json::json!({ "cycles": 500,
        "created": counts.region_created, "dropped": counts.region_dropped,
        "samples": samples, "note": "Hidden HWND, private process memory; not VRAM or a long-term leak proof" })
    );
}

#[test]
fn unchanged_clips_keep_paths_and_brush_identity() {
    ensure_dispatcher().unwrap();
    let compositor = Compositor::new().unwrap();
    let factory: ID2D1Factory =
        unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).unwrap() };
    let mut region = model(0);
    let mut visual = RegionVisual::new(&compositor, &factory, region.clone()).unwrap();
    let original: Vec<_> = visual
        .clips
        .iter()
        .map(|clip| clip.Path().unwrap())
        .collect();
    let brush = visual._brush.clone();
    region.clips[0].x += 1.0;
    visual
        .update(&compositor, &factory, region.clone())
        .unwrap();
    assert_ne!(visual.clips[0].Path().unwrap(), original[0]);
    assert_eq!(visual.clips[1].Path().unwrap(), original[1]);
    assert_eq!(visual._brush, brush);
    region.clips.push(region.clips[0].clone());
    visual
        .update(&compositor, &factory, region.clone())
        .unwrap();
    assert_eq!(visual.clips.len(), 5);
    for clip in &visual.clips {
        assert!(clip.Path().is_ok());
    }
    region.clips.truncate(2);
    visual.update(&compositor, &factory, region).unwrap();
    assert_eq!(visual.clips.len(), 2);
    assert_eq!(visual._brush, brush);
}

#[test]
fn native_visibility_gate_cannot_be_undone_by_geometry_updates() {
    use windows::{
        Win32::UI::WindowsAndMessaging::{CreateWindowExW, DestroyWindow, WS_POPUP},
        core::w,
    };
    ensure_dispatcher().unwrap();
    let hwnd = unsafe {
        CreateWindowExW(
            Default::default(),
            w!("STATIC"),
            w!("Native blur gate test"),
            WS_POPUP,
            0,
            0,
            320,
            200,
            None,
            None,
            None,
            None,
        )
        .unwrap()
    };
    struct HiddenWindow(HWND);
    impl Drop for HiddenWindow {
        fn drop(&mut self) {
            let _ = unsafe { DestroyWindow(self.0) };
        }
    }
    let _window = HiddenWindow(hwnd);
    let mut host = NativeBlurHost::new(hwnd.0 as isize).unwrap();
    host.update(vec![model(0)], 1.0).unwrap();
    host.arm_hide(std::time::Duration::from_millis(80)).unwrap();
    host.arm_hide(std::time::Duration::ZERO).unwrap();
    assert_eq!(host.root.Opacity().unwrap(), 0.0);
    let mut region = model(0);
    region.clips[0].x += 5.0;
    host.update(vec![region], 1.0).unwrap();
    assert_eq!(host.root.Opacity().unwrap(), 0.0);
}
