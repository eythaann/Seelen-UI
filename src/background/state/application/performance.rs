use std::sync::{atomic::AtomicU8, LazyLock};

use seelen_core::{handlers::SeelenEvent, state::PerformanceMode, system_state::PowerMode};
use tauri::Listener;

use crate::{
    app::{emit_to_webviews, get_app_handle},
    hook::HookManager,
    modules::power::infrastructure::{get_batteries, get_power_mode, get_power_status},
    state::application::FULL_STATE,
    windows_api::window::{event::WinEvent, Window},
};

pub static PERFORMANCE_MODE: LazyLock<Optimizations> = LazyLock::new(|| {
    let optimizations = Optimizations::create();
    optimizations.init();
    optimizations
});

pub struct Optimizations {
    inner: AtomicU8,
}

impl Optimizations {
    fn create() -> Self {
        Self {
            inner: AtomicU8::new(0),
        }
    }

    pub fn load(&self) -> PerformanceMode {
        let stored = self.inner.load(std::sync::atomic::Ordering::Acquire);
        PerformanceMode::from(stored)
    }

    fn init(&self) {
        let initial = calculate_current_perf_mode();
        let initial_u8: u8 = initial.into();

        log::info!("Initial performance mode: {initial:?}");
        self.inner
            .store(initial_u8, std::sync::atomic::Ordering::SeqCst);

        let handle = get_app_handle();
        handle.listen(SeelenEvent::PowerMode, |_| Self::check_for_changes());
        handle.listen(SeelenEvent::PowerStatus, |_| Self::check_for_changes());
        handle.listen(SeelenEvent::StateSettingsChanged, |_| {
            Self::check_for_changes()
        });

        HookManager::subscribe(|(event, _origin)| {
            if matches!(
                event,
                WinEvent::SystemForeground | WinEvent::SynDebouncedRectChange
            ) {
                Self::check_for_changes();
            }
        });
    }

    fn check_for_changes() {
        let stored = PERFORMANCE_MODE
            .inner
            .load(std::sync::atomic::Ordering::Acquire);
        let current = calculate_current_perf_mode();
        let current_u8: u8 = current.into();

        if current_u8 != stored {
            log::trace!("Seelen UI performance mode changed to {current:?}");
            PERFORMANCE_MODE
                .inner
                .store(current_u8, std::sync::atomic::Ordering::SeqCst);
            emit_to_webviews(SeelenEvent::StatePerformanceModeChanged, current);
        }
    }
}

fn calculate_current_perf_mode() -> PerformanceMode {
    let foreground = Window::get_foregrounded();
    if foreground.is_fullscreen() {
        return PerformanceMode::Extreme;
    }

    let power_mode = get_power_mode();
    if matches!(power_mode, PowerMode::GameMode | PowerMode::MixedReality) {
        return PerformanceMode::Extreme;
    }

    let guard = FULL_STATE.load();
    let config = &guard.settings.performance_mode;

    if matches!(
        power_mode,
        PowerMode::BatterySaver | PowerMode::BetterBattery
    ) {
        return config.on_energy_saver;
    }

    let power_status = get_power_status();
    let batteries = get_batteries();
    if !batteries.is_empty() && power_status.ac_line_status != 1 {
        return config.on_battery;
    }

    config.default
}
