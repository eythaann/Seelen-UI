use seelen_core::system_state::{Core, Disk, Gpu, Memory, NetworkStatistics};

use crate::error::Result;

/// OS contract of the system info module. Disks, network, memory and cores are read through
/// the cross-platform `sysinfo` crate, only what it doesn't cover is here.
pub trait SystemInfoSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Opens the performance counters. Called once.
    fn init(&mut self) -> Result<()>;

    /// Usage of the GPUs. Fails if the counters are not available.
    fn gpus(&self) -> Result<Vec<Gpu>>;
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum SystemInfoEvent {
    DisksChanged,
    NetworkChanged,
    MemoryChanged,
    CoresChanged,
    GpusChanged,
}

pub fn disks_changed(old: &[Disk], new: &[Disk]) -> bool {
    old.len() != new.len()
        || old.iter().zip(new).any(|(old, new)| {
            old.available_space != new.available_space
                || old.read_bytes != new.read_bytes
                || old.written_bytes != new.written_bytes
        })
}

pub fn network_changed(old: &[NetworkStatistics], new: &[NetworkStatistics]) -> bool {
    old.len() != new.len()
        || old
            .iter()
            .zip(new)
            .any(|(old, new)| old.received != new.received || old.transmitted != new.transmitted)
}

pub fn memory_changed(old: &Memory, new: &Memory) -> bool {
    old.free != new.free || old.swap_free != new.swap_free
}

pub fn cores_changed(old: &[Core], new: &[Core]) -> bool {
    old.len() != new.len() || old.iter().zip(new).any(|(old, new)| old.usage != new.usage)
}

pub fn gpus_changed(old: &[Gpu], new: &[Gpu]) -> bool {
    old.len() != new.len()
        || old.iter().zip(new).any(|(old, new)| {
            old.id != new.id
                || old.usage != new.usage
                || old.dedicated_memory_used != new.dedicated_memory_used
                || old.shared_memory_used != new.shared_memory_used
        })
}
