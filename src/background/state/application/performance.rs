use std::sync::{
    LazyLock,
    atomic::{AtomicU8, Ordering},
};

use seelen_core::{state::PerformanceMode, system_state::PowerMode};

use crate::{
    platform::{
        hook::{HookManager, WindowEvent},
        window::Window,
    },
    state::domain::StateEvent,
    system_modules::power::{PowerEvent, PowerManager},
};

use super::{FULL_STATE, StateManager};

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
        let stored = self.inner.load(Ordering::Acquire);
        PerformanceMode::from(stored)
    }

    fn init(&self) {
        let initial = calculate_current_perf_mode();
        let initial_u8: u8 = initial.into();

        log::info!("Initial performance mode: {initial:?}");
        self.inner.store(initial_u8, Ordering::SeqCst);

        PowerManager::subscribe(|event| {
            if matches!(
                event,
                PowerEvent::ModeChanged { .. }
                    | PowerEvent::StatusChanged { .. }
                    | PowerEvent::BatteriesChanged { .. }
            ) {
                Self::check_for_changes();
            }
        });

        StateManager::subscribe(|event| {
            if matches!(event, StateEvent::SettingsChanged { .. }) {
                Self::check_for_changes();
            }
        });

        HookManager::subscribe(|(event, _origin)| {
            if matches!(event, WindowEvent::Foregrounded | WindowEvent::RectChanged) {
                Self::check_for_changes();
            }
        });
    }

    fn check_for_changes() {
        let stored = PERFORMANCE_MODE.inner.load(Ordering::Acquire);
        let current = calculate_current_perf_mode();
        let current_u8: u8 = current.into();

        if current_u8 != stored {
            log::trace!("Seelen UI performance mode changed to {current:?}");
            PERFORMANCE_MODE.inner.store(current_u8, Ordering::SeqCst);
            StateManager::send(StateEvent::PerformanceModeChanged { mode: current });
        }
    }
}

fn calculate_current_perf_mode() -> PerformanceMode {
    let foreground = Window::get_foregrounded();
    if foreground.is_fullscreen() {
        return PerformanceMode::Extreme;
    }

    let power = PowerManager::instance();
    let power_mode = power.mode();
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

    if !power.batteries().is_empty() && power.status().ac_line_status != 1 {
        return config.on_battery;
    }

    config.default
}
