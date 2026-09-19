use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};

use parking_lot::Mutex;
use seelen_core::system_state::MonitorBrightness;
use wmi::WMIConnection;

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    modules::monitors::{
        application::{MonitorManager, MonitorManagerEvent},
        brightness::domain::{
            WmiMonitorBrightness, WmiMonitorBrightnessEvent, WmiMonitorBrightnessMethods,
            WmiSetBrightnessPayload,
        },
    },
    utils::lock_free::SyncVec,
    windows_api::{
        MonitorEnumerator,
        monitor::brightness::{DdcciBrightnessValues, PhysicalMonitorHandle},
    },
};

/// `WBEM_E_NOT_SUPPORTED`: the provider has no instances to offer on this hardware.
const WBEM_E_NOT_SUPPORTED: i32 = 0x8004100C_u32 as i32;

/// DDC/CI has no change notifications, so external monitors are polled at this interval to
/// pick up changes made from the monitor's own OSD. Each read costs one I2C round trip
/// (~60 ms on a healthy monitor) on a background thread.
const DDCCI_POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Serializes every DDC/CI transaction (polling and writes) *together with* the publication
/// of its result into `BrightnessManager::ddcci`. Holding the lock across both steps means a
/// read that started before a write can never overwrite the written value afterwards.
/// Concurrent transactions over the same I2C bus can also corrupt each other.
static DDCCI_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone)]
pub enum BrightnessManagerEvent {
    Changed(Vec<MonitorBrightness>),
}

event_manager!(BrightnessManager, BrightnessManagerEvent);

/// One physical panel reachable through DDC/CI. Brightness is exposed to the UI as a
/// percentage regardless of the raw range reported by the panel.
#[derive(Debug, Clone)]
struct DdcciMonitor {
    /// Stable monitor id, suffixed with `#<index>` for the extra panels of a cloned group.
    instance_name: String,
    handle: Arc<PhysicalMonitorHandle>,
    values: DdcciBrightnessValues,
}

impl DdcciMonitor {
    fn percent(&self) -> u8 {
        let DdcciBrightnessValues { min, current, max } = self.values;
        let span = max.saturating_sub(min).max(1);
        let current = current.clamp(min, max) - min;
        ((current as u64 * 100 + span as u64 / 2) / span as u64) as u8
    }

    fn raw_from_percent(&self, percent: u8) -> u32 {
        let DdcciBrightnessValues { min, max, .. } = self.values;
        let span = max.saturating_sub(min) as u64;
        min + ((percent.min(100) as u64 * span + 50) / 100) as u32
    }
}

impl From<&DdcciMonitor> for MonitorBrightness {
    fn from(m: &DdcciMonitor) -> Self {
        MonitorBrightness {
            instance_name: m.instance_name.clone(),
            current_brightness: m.percent(),
            levels: 101,
            available_levels: (0..=100).collect(),
            active: true,
        }
    }
}

impl From<WmiMonitorBrightness> for MonitorBrightness {
    fn from(b: WmiMonitorBrightness) -> Self {
        MonitorBrightness {
            instance_name: b.instance_name,
            current_brightness: b.current_brightness,
            levels: b.levels,
            available_levels: b.level,
            active: b.active,
        }
    }
}

enum DdcciWorkerMessage {
    /// A display was added/removed, re-enumerate monitors.
    Rescan,
}

pub struct BrightnessManager {
    /// Internal displays (laptops), managed through WMI.
    wmi: SyncVec<WmiMonitorBrightness>,
    /// External displays, managed through DDC/CI. Written only while holding `DDCCI_LOCK`.
    ddcci: SyncVec<DdcciMonitor>,
    ddcci_tx: crossbeam_channel::Sender<DdcciWorkerMessage>,
}

impl BrightnessManager {
    pub fn instance() -> &'static Self {
        static INSTANCE: LazyLock<BrightnessManager> = LazyLock::new(|| {
            let m = BrightnessManager::new();
            m.init_wmi().log_error();
            m.init_ddcci();
            m
        });
        &INSTANCE
    }

    fn new() -> Self {
        let (ddcci_tx, ddcci_rx) = crossbeam_channel::unbounded();
        std::thread::spawn(move || Self::ddcci_worker(ddcci_rx));
        Self {
            wmi: SyncVec::new(),
            ddcci: SyncVec::new(),
            ddcci_tx,
        }
    }

    fn init_wmi(&self) -> Result<()> {
        let wmi = WMIConnection::with_namespace_path("ROOT\\WMI")?;

        let brightness: Vec<WmiMonitorBrightness> = match wmi.query() {
            Ok(brightness) => brightness,
            // No monitor exposes brightness through WMI (the usual case for desktops with
            // external displays). There is nothing to track, so don't report it as an error.
            Err(wmi::WMIError::HResultError { hres }) if hres == WBEM_E_NOT_SUPPORTED => {
                log::debug!("Monitor brightness is not supported through WMI on this system");
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        };
        self.wmi.replace(brightness);

        std::thread::spawn(move || {
            let wmi = WMIConnection::with_namespace_path("ROOT\\WMI")?;
            for event in wmi.notification::<WmiMonitorBrightnessEvent>()? {
                let Ok(_event) = event else {
                    continue;
                };

                let brightness: Vec<WmiMonitorBrightness> = wmi.query()?;
                let manager = BrightnessManager::instance();
                manager.wmi.replace(brightness);
                manager.emit_changed();
            }
            Result::Ok(())
        });
        Ok(())
    }

    fn init_ddcci(&self) {
        MonitorManager::subscribe(|event| match event {
            MonitorManagerEvent::ViewAdded(_)
            | MonitorManagerEvent::ViewRemoved(_)
            | MonitorManagerEvent::ViewsChanged => {
                BrightnessManager::instance().request_ddcci_rescan();
            }
        });
        self.request_ddcci_rescan();
    }

    fn request_ddcci_rescan(&self) {
        // the worker owns the receiver for the whole process lifetime, so this can't fail
        let _ = self.ddcci_tx.send(DdcciWorkerMessage::Rescan);
    }

    /// Background loop for DDC/CI: enumerates monitors on demand and polls the known ones.
    /// `instance()` must not be called from here until the first message arrives, since the
    /// worker is spawned while the singleton is still being constructed.
    fn ddcci_worker(rx: crossbeam_channel::Receiver<DdcciWorkerMessage>) {
        // block until init sends the first rescan, i.e. until the singleton exists
        if rx.recv().is_err() {
            return;
        }
        Self::ddcci_scan();
        loop {
            match rx.recv_timeout(DDCCI_POLL_INTERVAL) {
                Ok(DdcciWorkerMessage::Rescan) => {
                    // collapse bursts of display events into a single scan
                    while rx.try_recv().is_ok() {}
                    Self::ddcci_scan();
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => Self::ddcci_poll(),
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => return,
            }
        }
    }

    /// Reads the brightness of a panel and validates the answer. Panels that don't speak
    /// DDC/CI fail the read; panels that answer nonsense are treated the same way, since
    /// writing to them could leave the panel in a bad state. Caller must hold `DDCCI_LOCK`.
    fn ddcci_read(handle: &PhysicalMonitorHandle) -> Result<DdcciBrightnessValues> {
        let values = handle.ddcci_get_brightness()?;
        if values.max <= values.min || values.current < values.min || values.current > values.max {
            return Err(format!("panel reported an invalid brightness range: {values:?}").into());
        }
        Ok(values)
    }

    fn ddcci_scan() {
        let monitors = match MonitorEnumerator::enumerate_win32() {
            Ok(monitors) => monitors,
            Err(err) => {
                log::warn!("Failed to enumerate monitors for DDC/CI brightness: {err}");
                return;
            }
        };

        let guard = DDCCI_LOCK.lock();
        let mut found = Vec::new();
        for monitor in monitors {
            let Ok((id, name)) = monitor.get_stable_info() else {
                continue;
            };
            let Ok(panels) = monitor.physical_monitors() else {
                continue;
            };
            for (index, handle) in panels.into_iter().enumerate() {
                let instance_name = if index == 0 {
                    id.0.clone()
                } else {
                    format!("{}#{index}", id.0)
                };
                let description = handle.description();
                match Self::ddcci_read(&handle) {
                    Ok(values) => {
                        log::debug!(
                            "DDC/CI brightness available on {name} / {description} ({instance_name}): {values:?}"
                        );
                        found.push(DdcciMonitor {
                            instance_name,
                            handle: Arc::new(handle),
                            values,
                        });
                    }
                    Err(err) => {
                        log::debug!(
                            "DDC/CI brightness unavailable on {name} / {description} ({instance_name}): {err}"
                        );
                    }
                }
            }
        }
        found.sort_by(|a, b| a.instance_name.cmp(&b.instance_name));

        let manager = Self::instance();
        let previous = manager.ddcci.to_vec();
        let changed = previous.len() != found.len()
            || previous
                .iter()
                .zip(&found)
                .any(|(a, b)| a.instance_name != b.instance_name || a.values != b.values);
        manager.ddcci.replace(found);
        drop(guard);

        if changed {
            manager.emit_changed();
        }
    }

    /// Re-reads the known DDC/CI panels to catch changes made from the monitor's OSD.
    /// A failed read (monitor asleep or unplugged) keeps the last known value; hotplug is
    /// handled by the rescan triggered from `MonitorManager`.
    fn ddcci_poll() {
        let manager = Self::instance();
        let guard = DDCCI_LOCK.lock();
        let mut changed = false;
        for known in manager.ddcci.to_vec() {
            let Ok(values) = Self::ddcci_read(&known.handle) else {
                continue;
            };
            if values != known.values {
                changed = true;
                manager.ddcci.for_each(|m| {
                    if m.instance_name == known.instance_name {
                        m.values = values;
                    }
                });
            }
        }
        drop(guard);

        if changed {
            manager.emit_changed();
        }
    }

    fn emit_changed(&self) {
        Self::send(BrightnessManagerEvent::Changed(self.get_all_brightness()));
    }

    pub fn get_all_brightness(&self) -> Vec<MonitorBrightness> {
        let mut all: Vec<MonitorBrightness> =
            self.wmi.to_vec().into_iter().map(Into::into).collect();
        all.extend(self.ddcci.map(|m| MonitorBrightness::from(&*m)));
        all
    }

    pub fn set_brightness(&self, instance_name: &str, level: u8) -> Result<()> {
        if self.ddcci.any(|m| m.instance_name == instance_name) {
            return self.set_ddcci_brightness(instance_name, level);
        }
        self.set_wmi_brightness(instance_name, level)
    }

    fn set_ddcci_brightness(&self, instance_name: &str, percent: u8) -> Result<()> {
        let guard = DDCCI_LOCK.lock();
        // re-resolve under the lock: a rescan may have replaced the handles meanwhile
        let monitor = self
            .ddcci
            .find_and_clone(|m| m.instance_name == instance_name)
            .ok_or("Instance not found")?;
        let raw = monitor.raw_from_percent(percent);
        monitor.handle.ddcci_set_brightness(raw)?;
        self.ddcci.for_each(|m| {
            if m.instance_name == instance_name {
                m.values.current = raw;
            }
        });
        drop(guard);

        self.emit_changed();
        Ok(())
    }

    fn set_wmi_brightness(&self, instance_name: &str, level: u8) -> Result<()> {
        let wmi = WMIConnection::with_namespace_path("ROOT\\WMI")?;

        let instances = wmi.query::<WmiMonitorBrightnessMethods>()?;

        let obj = instances
            .into_iter()
            .find(|v| v.instance_name == instance_name)
            .ok_or("Instance not found")?;

        wmi.exec_instance_method::<WmiMonitorBrightnessMethods, ()>(
            obj.__path,
            "WmiSetBrightness",
            WmiSetBrightnessPayload {
                timeout: 0,
                brightness: level,
            },
        )?;
        Ok(())
    }
}
