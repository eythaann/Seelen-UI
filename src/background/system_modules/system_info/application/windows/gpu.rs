use std::collections::HashMap;

use seelen_core::system_state::Gpu;
use windows::{
    Win32::{
        Graphics::Dxgi::{CreateDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE, IDXGIFactory1},
        System::Performance::{
            PDH_CSTATUS_NEW_DATA, PDH_CSTATUS_VALID_DATA, PDH_FMT_COUNTERVALUE_ITEM_W,
            PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY, PDH_MORE_DATA, PDH_NO_DATA,
            PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData,
            PdhGetFormattedCounterArrayW, PdhOpenQueryW,
        },
    },
    core::{PCWSTR, w},
};

use crate::error::Result;

// Same counters used by the Task Manager
const ENGINE_UTILIZATION: PCWSTR = w!("\\GPU Engine(*)\\Utilization Percentage");
const DEDICATED_USAGE: PCWSTR = w!("\\GPU Adapter Memory(*)\\Dedicated Usage");
const SHARED_USAGE: PCWSTR = w!("\\GPU Adapter Memory(*)\\Shared Usage");

/// Reads the usage of the GPUs via PDH performance counters, vendor agnostic.
pub struct GpuMonitor {
    query: PDH_HQUERY,
    engine_utilization: PDH_HCOUNTER,
    dedicated_usage: PDH_HCOUNTER,
    shared_usage: PDH_HCOUNTER,
    adapters: Vec<Adapter>,
}

// PDH handles are only accessed behind the SystemInfo mutex
unsafe impl Send for GpuMonitor {}

impl GpuMonitor {
    pub fn new() -> Result<Self> {
        let mut query = PDH_HQUERY::default();
        pdh_check(unsafe { PdhOpenQueryW(PCWSTR::null(), 0, &mut query) })?;

        // created before adding the counters so the query is closed on drop if any of them fails
        let mut monitor = Self {
            query,
            engine_utilization: PDH_HCOUNTER::default(),
            dedicated_usage: PDH_HCOUNTER::default(),
            shared_usage: PDH_HCOUNTER::default(),
            adapters: Adapter::enumerate()?,
        };
        monitor.engine_utilization = monitor.add_counter(ENGINE_UTILIZATION)?;
        monitor.dedicated_usage = monitor.add_counter(DEDICATED_USAGE)?;
        monitor.shared_usage = monitor.add_counter(SHARED_USAGE)?;

        // rate counters (utilization) need two samples to be computed
        pdh_check(unsafe { PdhCollectQueryData(monitor.query) })?;
        Ok(monitor)
    }

    fn add_counter(&self, path: PCWSTR) -> Result<PDH_HCOUNTER> {
        let mut counter = PDH_HCOUNTER::default();
        pdh_check(unsafe { PdhAddEnglishCounterW(self.query, path, 0, &mut counter) })?;
        Ok(counter)
    }

    pub fn gpus(&mut self) -> Result<Vec<Gpu>> {
        pdh_check(unsafe { PdhCollectQueryData(self.query) })?;

        // instance: `pid_1234_luid_0x00000000_0x0000d1b2_phys_0_eng_0_engtype_3d`
        // the same engine is reported once per process, so we sum them by engine
        let mut engines: HashMap<String, f64> = HashMap::new();
        for (instance, value) in read_counter(self.engine_utilization)? {
            if let Some(engine) = engine_key(&instance) {
                *engines.entry(engine.to_string()).or_default() += value;
            }
        }

        // instance: `luid_0x00000000_0x0000d1b2_phys_0`
        let dedicated = read_counter(self.dedicated_usage)?;
        let shared = read_counter(self.shared_usage)?;

        // an adapter was connected (eGPU, driver reinstall, etc)
        let has_unknown = dedicated
            .iter()
            .any(|(instance, _)| !self.adapters.iter().any(|a| instance.starts_with(&a.luid)));
        if has_unknown {
            self.adapters = Adapter::enumerate()?;
        }

        let sum_by_adapter = |values: &[(String, f64)], luid: &str| -> u64 {
            values
                .iter()
                .filter(|(instance, _)| instance.starts_with(luid))
                .map(|(_, value)| *value as u64)
                .sum()
        };

        Ok(self
            .adapters
            .iter()
            .map(|adapter| {
                // like the Task Manager, the adapter usage is the one of its busiest engine
                let usage = engines
                    .iter()
                    .filter(|(engine, _)| engine.starts_with(&adapter.luid))
                    .map(|(_, value)| *value)
                    .fold(0.0, f64::max)
                    .min(100.0);

                Gpu {
                    id: adapter.luid.clone(),
                    name: adapter.name.clone(),
                    usage: usage as f32,
                    dedicated_memory_total: adapter.dedicated_memory,
                    dedicated_memory_used: sum_by_adapter(&dedicated, &adapter.luid),
                    shared_memory_total: adapter.shared_memory,
                    shared_memory_used: sum_by_adapter(&shared, &adapter.luid),
                }
            })
            .collect())
    }
}

impl Drop for GpuMonitor {
    fn drop(&mut self) {
        unsafe { PdhCloseQuery(self.query) };
    }
}

struct Adapter {
    /// lowercase, formatted as in the PDH instance names: `luid_0x00000000_0x0000d1b2`
    luid: String,
    name: String,
    dedicated_memory: u64,
    shared_memory: u64,
}

impl Adapter {
    fn enumerate() -> Result<Vec<Self>> {
        let factory: IDXGIFactory1 = unsafe { CreateDXGIFactory1()? };
        let mut adapters = Vec::new();
        let mut index = 0;
        // EnumAdapters1 fails with DXGI_ERROR_NOT_FOUND when there are no more adapters
        while let Ok(adapter) = unsafe { factory.EnumAdapters1(index) } {
            index += 1;
            let desc = unsafe { adapter.GetDesc1()? };
            if desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0 {
                continue;
            }
            let name_len = desc
                .Description
                .iter()
                .position(|c| *c == 0)
                .unwrap_or(desc.Description.len());
            adapters.push(Self {
                luid: format!(
                    "luid_0x{:08x}_0x{:08x}",
                    desc.AdapterLuid.HighPart, desc.AdapterLuid.LowPart
                ),
                name: String::from_utf16_lossy(&desc.Description[..name_len]),
                dedicated_memory: desc.DedicatedVideoMemory as u64,
                shared_memory: desc.SharedSystemMemory as u64,
            });
        }
        Ok(adapters)
    }
}

fn pdh_check(status: u32) -> Result<()> {
    if status == PDH_CSTATUS_VALID_DATA {
        return Ok(());
    }
    Err(format!("PDH call failed with status 0x{status:08X}").into())
}

/// Returns the lowercased instance names with their values, skipping the invalid ones.
fn read_counter(counter: PDH_HCOUNTER) -> Result<Vec<(String, f64)>> {
    loop {
        let mut buffer_size = 0u32;
        let mut item_count = 0u32;
        let status = unsafe {
            PdhGetFormattedCounterArrayW(
                counter,
                PDH_FMT_DOUBLE,
                &mut buffer_size,
                &mut item_count,
                None,
            )
        };
        match status {
            PDH_MORE_DATA => {}
            PDH_NO_DATA => return Ok(Vec::new()),
            _ => {
                pdh_check(status)?;
                return Ok(Vec::new());
            }
        }

        // buffer_size is in bytes and also includes the instance name strings
        let item_size = std::mem::size_of::<PDH_FMT_COUNTERVALUE_ITEM_W>();
        let mut buffer = vec![
            PDH_FMT_COUNTERVALUE_ITEM_W::default();
            (buffer_size as usize).div_ceil(item_size)
        ];
        let status = unsafe {
            PdhGetFormattedCounterArrayW(
                counter,
                PDH_FMT_DOUBLE,
                &mut buffer_size,
                &mut item_count,
                Some(buffer.as_mut_ptr()),
            )
        };
        // instances changed between both calls (a process started using the GPU)
        if status == PDH_MORE_DATA {
            continue;
        }
        pdh_check(status)?;

        return Ok(buffer[..item_count as usize]
            .iter()
            .filter(|item| {
                item.FmtValue.CStatus == PDH_CSTATUS_VALID_DATA
                    || item.FmtValue.CStatus == PDH_CSTATUS_NEW_DATA
            })
            .filter_map(|item| {
                let name = unsafe { item.szName.to_string() }.ok()?;
                let value = unsafe { item.FmtValue.Anonymous.doubleValue };
                Some((name.to_lowercase(), value))
            })
            .collect());
    }
}

/// `pid_1234_luid_0x00000000_0x0000d1b2_phys_0_eng_0_engtype_3d` -> `luid_0x00000000_0x0000d1b2_phys_0_eng_0`
fn engine_key(instance: &str) -> Option<&str> {
    let start = instance.find("luid_")?;
    let end = instance.find("_engtype_").unwrap_or(instance.len());
    instance.get(start..end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_key_strips_pid_and_engine_type() {
        assert_eq!(
            engine_key("pid_1234_luid_0x00000000_0x0000d1b2_phys_0_eng_0_engtype_3d"),
            Some("luid_0x00000000_0x0000d1b2_phys_0_eng_0")
        );
        assert_eq!(engine_key("invalid"), None);
    }
}
