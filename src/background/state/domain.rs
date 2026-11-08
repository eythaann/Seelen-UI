use seelen_core::state::{PerformanceMode, Settings};

#[derive(Debug, Clone)]
pub enum StateEvent {
    SettingsChanged { settings: Box<Settings> },
    PerformanceModeChanged { mode: PerformanceMode },
}
