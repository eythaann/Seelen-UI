#[cfg(windows)]
mod windows;

use std::{sync::LazyLock, time::Duration};

use seelen_core::system_state::{Core, Disk, Gpu, Memory, NetworkStatistics};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    state::application::FULL_STATE,
    utils::lock_free::TracedMutex,
};

use super::domain::{
    SystemInfoEvent, SystemInfoSystem, cores_changed, disks_changed, gpus_changed, memory_changed,
    network_changed,
};

#[cfg(windows)]
type Platform = windows::WindowsSystemInfo;

/// Polling pause while the session is not interactive.
const IDLE_INTERVAL: Duration = Duration::from_secs(5);

pub struct SystemInfo {
    platform: Platform,
    disks: TracedMutex<sysinfo::Disks>,
    networks: TracedMutex<sysinfo::Networks>,
    system: TracedMutex<sysinfo::System>,
    // Last values, used to answer queries and to only notify changes
    last_disks: TracedMutex<Vec<Disk>>,
    last_networks: TracedMutex<Vec<NetworkStatistics>>,
    last_memory: TracedMutex<Memory>,
    last_cores: TracedMutex<Vec<Core>>,
    last_gpus: TracedMutex<Vec<Gpu>>,
}

event_manager!(SystemInfo, SystemInfoEvent);

impl SystemInfo {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<SystemInfo> = LazyLock::new(|| {
            let mut m = SystemInfo::create();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
            disks: TracedMutex::new(sysinfo::Disks::new()),
            networks: TracedMutex::new(sysinfo::Networks::new()),
            system: TracedMutex::new(sysinfo::System::new()),
            last_disks: TracedMutex::new(Vec::new()),
            last_networks: TracedMutex::new(Vec::new()),
            last_memory: TracedMutex::new(Memory::default()),
            last_cores: TracedMutex::new(Vec::new()),
            last_gpus: TracedMutex::new(Vec::new()),
        }
    }

    fn init(&mut self) -> Result<()> {
        // the GPU counters are optional, the rest still works without them
        self.platform.init().log_error();

        *self.last_disks.lock() = self.read_disks();
        *self.last_networks.lock() = self.read_network();
        *self.last_memory.lock() = self.read_memory();
        *self.last_cores.lock() = self.read_cores();
        if let Some(gpus) = self.read_gpus() {
            *self.last_gpus.lock() = gpus;
        }

        std::thread::spawn(|| {
            loop {
                let manager = SystemInfo::instance();
                if !crate::platform::session::is_interactive() {
                    std::thread::sleep(IDLE_INTERVAL);
                    continue;
                }
                let interval = FULL_STATE.load().settings.polling_interval;
                std::thread::sleep(Duration::from_secs(interval));
                manager.check_and_emit_changes();
            }
        });
        Ok(())
    }

    fn check_and_emit_changes(&self) {
        update_if(&self.last_disks, self.read_disks(), |o, n| {
            disks_changed(o, n)
        })
        .then(|| Self::send(SystemInfoEvent::DisksChanged));
        update_if(&self.last_networks, self.read_network(), |o, n| {
            network_changed(o, n)
        })
        .then(|| Self::send(SystemInfoEvent::NetworkChanged));
        update_if(&self.last_memory, self.read_memory(), memory_changed)
            .then(|| Self::send(SystemInfoEvent::MemoryChanged));
        update_if(&self.last_cores, self.read_cores(), |o, n| {
            cores_changed(o, n)
        })
        .then(|| Self::send(SystemInfoEvent::CoresChanged));
        if let Some(gpus) = self.read_gpus() {
            update_if(&self.last_gpus, gpus, |o, n| gpus_changed(o, n))
                .then(|| Self::send(SystemInfoEvent::GpusChanged));
        }
    }

    fn read_disks(&self) -> Vec<Disk> {
        let interval = FULL_STATE.load().settings.polling_interval;
        let mut guard = self.disks.lock();
        guard.refresh(true);
        guard
            .iter()
            .map(|disk| {
                let usage = disk.usage();
                Disk {
                    name: disk.name().to_string_lossy().to_string(),
                    file_system: disk.file_system().to_string_lossy().to_string(),
                    mount_point: disk.mount_point().to_path_buf(),
                    total_space: disk.total_space(),
                    available_space: disk.available_space(),
                    is_removable: disk.is_removable(),
                    read_bytes: usage.read_bytes / interval,
                    written_bytes: usage.written_bytes / interval,
                }
            })
            .collect()
    }

    fn read_network(&self) -> Vec<NetworkStatistics> {
        let interval = FULL_STATE.load().settings.polling_interval;
        let mut guard = self.networks.lock();
        guard.refresh(true);
        guard
            .iter()
            .map(|(name, network)| NetworkStatistics {
                name: name.clone(),
                received: network.received() / interval,
                transmitted: network.transmitted() / interval,
                packets_received: network.packets_received() / interval,
                packets_transmitted: network.packets_transmitted() / interval,
            })
            .collect()
    }

    fn read_memory(&self) -> Memory {
        let mut system = self.system.lock();
        system.refresh_memory();
        Memory {
            total: system.total_memory(),
            free: system.free_memory(),
            swap_total: system.total_swap(),
            swap_free: system.free_swap(),
        }
    }

    fn read_cores(&self) -> Vec<Core> {
        let mut system = self.system.lock();
        system.refresh_cpu_all();
        system
            .cpus()
            .iter()
            .map(|cpu| Core {
                name: cpu.name().to_owned(),
                brand: cpu.brand().to_owned(),
                usage: cpu.cpu_usage(),
                frequency: cpu.frequency(),
            })
            .collect()
    }

    /// Returns None on failure, to keep the last known value
    fn read_gpus(&self) -> Option<Vec<Gpu>> {
        match self.platform.gpus() {
            Ok(gpus) => Some(gpus),
            Err(err) => {
                log::debug!("Failed to read GPU usage: {err:?}");
                None
            }
        }
    }

    pub fn disks(&self) -> Vec<Disk> {
        self.last_disks.lock().clone()
    }

    pub fn network(&self) -> Vec<NetworkStatistics> {
        self.last_networks.lock().clone()
    }

    pub fn memory(&self) -> Memory {
        self.last_memory.lock().clone()
    }

    pub fn cores(&self) -> Vec<Core> {
        self.last_cores.lock().clone()
    }

    pub fn gpus(&self) -> Vec<Gpu> {
        self.last_gpus.lock().clone()
    }
}

/// Replaces the last value when `changed`, returning whether it did.
fn update_if<T>(last: &TracedMutex<T>, current: T, changed: impl Fn(&T, &T) -> bool) -> bool {
    let mut last = last.lock();
    if !changed(&last, &current) {
        return false;
    }
    *last = current;
    true
}
