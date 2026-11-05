#[cfg(windows)]
mod windows;

use std::sync::LazyLock;

use seelen_core::system_state::TrashBinInfo;

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    utils::lock_free::TracedMutex,
};

use super::domain::{TrashBinEvent, TrashBinSystem};

#[cfg(windows)]
type Platform = windows::WindowsTrashBin;

pub struct TrashBinManager {
    platform: Platform,
    /// Last known info, used to answer queries and to drop OS notifications without changes.
    info: TracedMutex<TrashBinInfo>,
}

event_manager!(TrashBinManager, TrashBinEvent);

impl TrashBinManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<TrashBinManager> = LazyLock::new(|| {
            let mut m = TrashBinManager::create();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
            info: TracedMutex::new(TrashBinInfo::default()),
        }
    }

    fn init(&mut self) -> Result<()> {
        self.platform.init(Self::on_platform_event)?;
        *self.info.lock() = self.platform.query()?;
        Ok(())
    }

    /// Updates the cached info and forwards the event only when the info really changed.
    fn on_platform_event(event: TrashBinEvent) {
        let TrashBinEvent::InfoChanged { info } = &event;
        {
            let mut current = Self::instance().info.lock();
            if *current == *info {
                return;
            }
            *current = info.clone();
        }
        Self::send(event);
    }

    pub fn info(&self) -> TrashBinInfo {
        self.info.lock().clone()
    }

    pub fn empty(&self) -> Result<()> {
        self.platform.empty()
    }
}
