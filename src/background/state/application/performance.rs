use std::sync::LazyLock;

use arc_swap::ArcSwap;
use seelen_core::{handlers::SeelenEvent, state::PerformanceMode, system_state::PowerMode};
use tauri::{Emitter, Listener};

use crate::{
    app::get_app_handle,
    error::{ErrorMap, ResultLogExt},
    hook::HookManager,
    modules::power::infrastructure::{get_batteries, get_power_mode, get_power_status},
    state::application::FULL_STATE,
    windows_api::window::{event::WinEvent, Window},
};

pub static PERFORMANCE_MODE: LazyLock<ArcSwap<PerformanceMode>> = LazyLock::new(|| {
    start_listeners();
    let perf_mode = get_perf_mode();
    log::info!("Performance mode: {perf_mode:?}");
    ArcSwap::from_pointee(perf_mode)
});

fn start_listeners() {
    let handle = get_app_handle();
    handle.listen(SeelenEvent::PowerMode, |_| check_for_changes());
    handle.listen(SeelenEvent::PowerStatus, |_| check_for_changes());
    handle.listen(SeelenEvent::StateSettingsChanged, |_| check_for_changes());

    HookManager::subscribe(|(event, _origin)| {
        if matches!(
            event,
            WinEvent::SystemForeground
                | WinEvent::SyntheticFullscreenStart
                | WinEvent::SyntheticFullscreenEnd
        ) {
            check_for_changes();
        }
    });
}

fn get_perf_mode() -> PerformanceMode {
    let foreground = Window::get_foregrounded();
    if foreground.get_cached_data().fullscreen && !foreground.is_seelen_overlay() {
        return PerformanceMode::Extreme;
    }

    let guard = FULL_STATE.load();
    let config = &guard.settings.performance_mode;

    let power_mode = get_power_mode();
    if power_mode == PowerMode::BatterySaver {
        return config.on_energy_saver;
    }

    let power_status = get_power_status();
    let batteries = get_batteries();
    if !batteries.is_empty() && power_status.ac_line_status != 1 {
        return config.on_battery;
    }

    config.default
}

fn check_for_changes() {
    let stored = PERFORMANCE_MODE.load_full();
    let current = get_perf_mode();
    if current != *stored {
        log::trace!("Seelen UI performance mode changed to {current:?}");
        PERFORMANCE_MODE.store(std::sync::Arc::new(current));
        get_app_handle()
            .emit(SeelenEvent::StatePerformanceModeChanged, current)
            .wrap_error()
            .log_error();
    }
}
