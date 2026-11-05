mod gpu;

use std::sync::atomic::Ordering;

use seelen_core::system_state::Gpu;

use crate::{
    error::Result, system_modules::system_info::domain::SystemInfoSystem,
    utils::lock_free::TracedMutex, windows_api::event_window::IS_INTERACTIVE_SESSION,
};

use self::gpu::GpuMonitor;

pub struct WindowsSystemInfo {
    /// None if the GPU performance counters are not available
    gpu: TracedMutex<Option<GpuMonitor>>,
}

impl SystemInfoSystem for WindowsSystemInfo {
    fn create() -> Self {
        Self {
            gpu: TracedMutex::new(None),
        }
    }

    fn init(&mut self) -> Result<()> {
        *self.gpu.lock() = Some(GpuMonitor::new()?);
        Ok(())
    }

    fn gpus(&self) -> Result<Vec<Gpu>> {
        match self.gpu.lock().as_mut() {
            Some(monitor) => monitor.gpus(),
            None => Err("GPU performance counters are not available".into()),
        }
    }

    fn is_session_interactive(&self) -> bool {
        IS_INTERACTIVE_SESSION.load(Ordering::Acquire)
    }
}
