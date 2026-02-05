pub mod tauri;

use std::sync::LazyLock;

use seelen_core::system_state::{Core, Disk, Memory, NetworkStatistics};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    utils::lock_free::TracedMutex,
};

pub struct SystemInfo {
    disks: TracedMutex<sysinfo::Disks>,
    networks: TracedMutex<sysinfo::Networks>,
    system: TracedMutex<sysinfo::System>,
    // Cache for change detection
    pub last_disks: TracedMutex<Vec<Disk>>,
    pub last_networks: TracedMutex<Vec<NetworkStatistics>>,
    pub last_memory: TracedMutex<Memory>,
    pub last_cores: TracedMutex<Vec<Core>>,
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum SystemInfoEvent {
    DisksChanged,
    NetworkChanged,
    MemoryChanged,
    CoresChanged,
}

event_manager!(SystemInfo, SystemInfoEvent);

impl SystemInfo {
    pub fn instance() -> &'static SystemInfo {
        static INSTANCE: LazyLock<SystemInfo> = LazyLock::new(|| {
            let mut instance = SystemInfo::new();
            instance.init().log_error();
            instance
        });
        &INSTANCE
    }

    fn new() -> Self {
        Self {
            disks: TracedMutex::new(sysinfo::Disks::new()),
            networks: TracedMutex::new(sysinfo::Networks::new()),
            system: TracedMutex::new(sysinfo::System::new()),
            // Cache for change detection
            last_disks: TracedMutex::new(Vec::new()),
            last_networks: TracedMutex::new(Vec::new()),
            last_memory: TracedMutex::new(Memory::default()),
            last_cores: TracedMutex::new(Vec::new()),
        }
    }

    fn init(&mut self) -> Result<()> {
        // Initialize first data
        *self.last_disks.lock() = self.disks();
        *self.last_networks.lock() = self.network();
        *self.last_memory.lock() = self.memory();
        *self.last_cores.lock() = self.cores();

        // Spawn monitoring thread
        std::thread::spawn(|| loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            SystemInfo::instance().check_and_emit_changes();
        });

        Ok(())
    }

    fn check_and_emit_changes(&self) {
        // Check disks (compare available_space)
        {
            let current_disks = self.disks();
            let mut last_disks = self.last_disks.lock();
            if Self::disks_changed(&last_disks, &current_disks) {
                *last_disks = current_disks;
                Self::send(SystemInfoEvent::DisksChanged);
            }
        }

        // Check network (compare received and transmitted)
        {
            let current_network = self.network();
            let mut last_network = self.last_networks.lock();
            if Self::network_changed(&last_network, &current_network) {
                *last_network = current_network;
                Self::send(SystemInfoEvent::NetworkChanged);
            }
        }

        {
            let current_memory = self.memory();
            let mut last_memory = self.last_memory.lock();
            if Self::memory_changed(&last_memory, &current_memory) {
                *last_memory = current_memory;
                Self::send(SystemInfoEvent::MemoryChanged);
            }
        }

        {
            let current_cores = self.cores();
            let mut last_cores = self.last_cores.lock();
            if Self::cores_changed(&last_cores, &current_cores) {
                *last_cores = current_cores;
                Self::send(SystemInfoEvent::CoresChanged);
            }
        }
    }

    fn disks_changed(old: &[Disk], new: &[Disk]) -> bool {
        if old.len() != new.len() {
            return true;
        }
        for (old_disk, new_disk) in old.iter().zip(new.iter()) {
            if old_disk.available_space != new_disk.available_space
                || old_disk.read_bytes != new_disk.read_bytes
                || old_disk.written_bytes != new_disk.written_bytes
            {
                return true;
            }
        }
        false
    }

    fn network_changed(old: &[NetworkStatistics], new: &[NetworkStatistics]) -> bool {
        if old.len() != new.len() {
            return true;
        }
        for (old_net, new_net) in old.iter().zip(new.iter()) {
            if old_net.received != new_net.received || old_net.transmitted != new_net.transmitted {
                return true;
            }
        }
        false
    }

    fn memory_changed(old: &Memory, new: &Memory) -> bool {
        old.free != new.free || old.swap_free != new.swap_free
    }

    fn cores_changed(old: &[Core], new: &[Core]) -> bool {
        if old.len() != new.len() {
            return true;
        }
        for (old_core, new_core) in old.iter().zip(new.iter()) {
            if old_core.usage != new_core.usage {
                return true;
            }
        }
        false
    }

    fn disks(&self) -> Vec<Disk> {
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
                    read_bytes: usage.read_bytes,
                    written_bytes: usage.written_bytes,
                }
            })
            .collect()
    }

    fn network(&self) -> Vec<NetworkStatistics> {
        let mut guard = self.networks.lock();
        guard.refresh(true);
        guard
            .iter()
            .map(|(name, network)| NetworkStatistics {
                name: name.clone(),
                received: network.received(),
                transmitted: network.transmitted(),
                packets_received: network.packets_received(),
                packets_transmitted: network.packets_transmitted(),
            })
            .collect()
    }

    fn memory(&self) -> Memory {
        let mut system = self.system.lock();
        system.refresh_memory();

        Memory {
            total: system.total_memory(),
            free: system.free_memory(),
            swap_total: system.total_swap(),
            swap_free: system.free_swap(),
        }
    }

    fn cores(&self) -> Vec<Core> {
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
}
