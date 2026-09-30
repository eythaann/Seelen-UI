use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};

use parking_lot::Mutex;
use seelen_core::system_state::MonitorId;
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
        monitor::{
            Monitor,
            brightness::{DdcciBrightnessValues, PhysicalMonitorHandle},
        },
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

/// Tracks the brightness of every panel, keyed by the stable id of the monitor showing it.
/// A monitor usually has a single panel, but a cloned/mirrored group has one per display.
pub struct BrightnessManager {
    /// Internal displays (laptops), managed through WMI.
    wmi: SyncVec<WmiPanel>,
    /// External displays, managed through DDC/CI. Written only while holding `DDCCI_LOCK`.
    ddcci: SyncVec<DdcciPanel>,
    worker_tx: crossbeam_channel::Sender<WorkerMessage>,
}

#[derive(Debug, Clone)]
pub enum BrightnessManagerEvent {
    Changed,
}

event_manager!(BrightnessManager, BrightnessManagerEvent);

impl BrightnessManager {
    pub fn instance() -> &'static Self {
        static INSTANCE: LazyLock<BrightnessManager> = LazyLock::new(|| {
            let m = BrightnessManager::new();
            m.init().log_error();
            m
        });
        &INSTANCE
    }

    fn new() -> Self {
        let (worker_tx, worker_rx) = crossbeam_channel::unbounded();
        std::thread::spawn(move || Self::worker(worker_rx));
        Self {
            wmi: SyncVec::new(),
            ddcci: SyncVec::new(),
            worker_tx,
        }
    }

    fn init(&self) -> Result<()> {
        MonitorManager::subscribe(|event| match event {
            MonitorManagerEvent::ViewAdded(_)
            | MonitorManagerEvent::ViewRemoved(_)
            | MonitorManagerEvent::ViewsChanged => {
                BrightnessManager::instance().request_rescan();
            }
        });
        self.request_rescan();
        self.listen_wmi()
    }

    /// WMI does notify brightness changes (keyboard keys, Windows settings, power plans).
    fn listen_wmi(&self) -> Result<()> {
        let wmi = WMIConnection::with_namespace_path("ROOT\\WMI")?;
        if Self::wmi_query(&wmi)?.is_none() {
            log::debug!("Monitor brightness is not supported through WMI on this system");
            return Ok(());
        }

        std::thread::spawn(move || {
            let wmi = WMIConnection::with_namespace_path("ROOT\\WMI")?;
            for event in wmi.notification::<WmiMonitorBrightnessEvent>()? {
                let Ok(event) = event else {
                    continue;
                };
                BrightnessManager::instance().on_wmi_event(event);
            }
            Result::Ok(())
        });
        Ok(())
    }

    fn on_wmi_event(&self, event: WmiMonitorBrightnessEvent) {
        let mut known = false;
        let mut changed = false;
        self.wmi.for_each(|panel| {
            if panel.instance_name == event.instance_name {
                known = true;
                changed |= panel.percent != event.brightness;
                panel.percent = event.brightness;
            }
        });

        if !known {
            self.request_rescan();
        } else if changed {
            self.emit_changed();
        }
    }

    fn request_rescan(&self) {
        // the worker owns the receiver for the whole process lifetime, so this can't fail
        let _ = self.worker_tx.send(WorkerMessage::Rescan);
    }

    /// Background loop: pairs the panels with the monitors on demand and polls the DDC/CI ones.
    /// `instance()` must not be called from here until the first message arrives, since the
    /// worker is spawned while the singleton is still being constructed.
    fn worker(rx: crossbeam_channel::Receiver<WorkerMessage>) {
        // block until init sends the first rescan, i.e. until the singleton exists
        if rx.recv().is_err() {
            return;
        }
        Self::scan();
        loop {
            match rx.recv_timeout(DDCCI_POLL_INTERVAL) {
                Ok(WorkerMessage::Rescan) => {
                    // collapse bursts of display events into a single scan
                    while rx.try_recv().is_ok() {}
                    Self::scan();
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => Self::ddcci_poll(),
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => return,
            }
        }
    }

    /// Enumerates the connected monitors and looks for the panels behind each of them.
    fn scan() {
        let monitors = match MonitorEnumerator::enumerate_win32() {
            Ok(monitors) => monitors,
            Err(err) => {
                log::warn!("Failed to enumerate monitors for brightness: {err}");
                return;
            }
        };
        let targets: Vec<ScanTarget> = monitors
            .into_iter()
            .filter_map(|monitor| {
                let (id, name) = monitor.get_stable_info().ok()?;
                Some(ScanTarget { monitor, id, name })
            })
            .collect();

        let manager = Self::instance();
        let wmi_changed = manager.wmi_scan(&targets);
        let ddcci_changed = manager.ddcci_scan(&targets);
        if wmi_changed || ddcci_changed {
            manager.emit_changed();
        }
    }

    /// Returns `None` when no monitor exposes brightness through WMI (the usual case for
    /// desktops with external displays), which is not an error.
    fn wmi_query(wmi: &WMIConnection) -> Result<Option<Vec<WmiMonitorBrightness>>> {
        match wmi.query() {
            Ok(instances) => Ok(Some(instances)),
            Err(wmi::WMIError::HResultError { hres }) if hres == WBEM_E_NOT_SUPPORTED => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// WMI identifies the panels by PnP device, so each instance is paired with the monitor
    /// scanning out to that device. Instances without a connected monitor (e.g. a panel still
    /// cached by the provider) are skipped.
    fn wmi_read(targets: &[ScanTarget]) -> Result<Vec<WmiPanel>> {
        let wmi = WMIConnection::with_namespace_path("ROOT\\WMI")?;
        let instances = Self::wmi_query(&wmi)?.unwrap_or_default();
        if instances.is_empty() {
            return Ok(Vec::new());
        }

        let devices: Vec<(&ScanTarget, Vec<String>)> = targets
            .iter()
            .map(|target| {
                let paths = target.monitor.target_device_paths().unwrap_or_default();
                let ids = paths.iter().map(|p| pnp_id_from_device_path(p)).collect();
                (target, ids)
            })
            .collect();

        let mut found = Vec::new();
        for instance in instances {
            let device_id = pnp_id_from_wmi_instance(&instance.instance_name);
            let Some((target, _)) = devices.iter().find(|(_, ids)| ids.contains(&device_id)) else {
                log::debug!(
                    "WMI brightness instance without a connected monitor: {}",
                    instance.instance_name
                );
                continue;
            };
            log::debug!(
                "WMI brightness available on {} ({}): {}%",
                target.name,
                target.id,
                instance.current_brightness
            );
            found.push(WmiPanel {
                monitor_id: target.id.clone(),
                instance_name: instance.instance_name,
                percent: instance.current_brightness.min(100),
            });
        }
        Ok(found)
    }

    /// Returns whether the known WMI panels changed.
    fn wmi_scan(&self, targets: &[ScanTarget]) -> bool {
        let found = match Self::wmi_read(targets) {
            Ok(found) => found,
            Err(err) => {
                log::warn!("Failed to read WMI brightness: {err}");
                return false;
            }
        };
        let changed = self.wmi.to_vec() != found;
        self.wmi.replace(found);
        changed
    }

    fn wmi_set(instance_name: &str, percent: u8) -> Result<()> {
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
                brightness: percent,
            },
        )?;
        Ok(())
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

    /// Returns whether the known DDC/CI panels changed.
    fn ddcci_scan(&self, targets: &[ScanTarget]) -> bool {
        let _guard = DDCCI_LOCK.lock();
        let mut found = Vec::new();
        for target in targets {
            let ScanTarget { monitor, id, name } = target;
            let Ok(panels) = monitor.physical_monitors() else {
                continue;
            };
            for handle in panels {
                let description = handle.description();
                match Self::ddcci_read(&handle) {
                    Ok(values) => {
                        log::debug!(
                            "DDC/CI brightness available on {name} / {description} ({id}): {values:?}"
                        );
                        found.push(DdcciPanel {
                            monitor_id: id.clone(),
                            handle: Arc::new(handle),
                            values,
                        });
                    }
                    Err(err) => {
                        log::debug!(
                            "DDC/CI brightness unavailable on {name} / {description} ({id}): {err}"
                        );
                    }
                }
            }
        }
        // stable sort: the panels of a cloned group keep their order
        found.sort_by(|a, b| a.monitor_id.0.cmp(&b.monitor_id.0));

        let previous = self.ddcci.to_vec();
        let changed = previous.len() != found.len()
            || previous
                .iter()
                .zip(&found)
                .any(|(a, b)| a.monitor_id != b.monitor_id || a.values != b.values);
        self.ddcci.replace(found);
        changed
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
                manager.ddcci.for_each(|panel| {
                    if Arc::ptr_eq(&panel.handle, &known.handle) {
                        panel.values = values;
                    }
                });
            }
        }
        drop(guard);

        if changed {
            manager.emit_changed();
        }
    }

    /// Returns whether the monitor has any DDC/CI panel.
    fn ddcci_set(&self, id: &MonitorId, percent: u8) -> Result<bool> {
        let guard = DDCCI_LOCK.lock();
        // resolve under the lock: a rescan may have replaced the handles meanwhile
        let panels: Vec<DdcciPanel> = self
            .ddcci
            .to_vec()
            .into_iter()
            .filter(|panel| &panel.monitor_id == id)
            .collect();

        for target in &panels {
            let raw = target.raw_from_percent(percent);
            target.handle.ddcci_set_brightness(raw)?;
            self.ddcci.for_each(|panel| {
                if Arc::ptr_eq(&panel.handle, &target.handle) {
                    panel.values.current = raw;
                }
            });
        }
        drop(guard);

        if !panels.is_empty() {
            self.emit_changed();
        }
        Ok(!panels.is_empty())
    }

    fn emit_changed(&self) {
        Self::send(BrightnessManagerEvent::Changed);
    }

    /// Brightness of the monitor as a percentage, `None` if it can't be controlled.
    pub fn get_brightness(&self, id: &MonitorId) -> Option<u8> {
        self.wmi
            .find_and_clone(|panel| &panel.monitor_id == id)
            .map(|panel| panel.percent)
            .or_else(|| {
                self.ddcci
                    .find_and_clone(|panel| &panel.monitor_id == id)
                    .map(|panel| panel.percent())
            })
    }

    /// Sets the brightness of every panel behind the monitor. `percent` goes from 0 to 100
    /// and is mapped to whatever range each panel works with.
    pub fn set_brightness(&self, id: &MonitorId, percent: u8) -> Result<()> {
        let percent = percent.min(100);

        let mut supported = false;
        for panel in self.wmi.to_vec() {
            if &panel.monitor_id == id {
                supported = true;
                // the cached value is updated by the WMI notification
                Self::wmi_set(&panel.instance_name, percent)?;
            }
        }
        supported |= self.ddcci_set(id, percent)?;

        if !supported {
            return Err("Monitor does not support brightness control".into());
        }
        Ok(())
    }
}

enum WorkerMessage {
    /// A display was added/removed, re-enumerate monitors.
    Rescan,
}

/// A connected monitor to look for panels on.
struct ScanTarget {
    monitor: Monitor,
    id: MonitorId,
    name: String,
}

/// One panel reachable through WMI, which already works with percentages.
#[derive(Debug, Clone, PartialEq, Eq)]
struct WmiPanel {
    monitor_id: MonitorId,
    /// e.g. `DISPLAY\BOE0900\4&10fd3ab1&0&UID265988_0`
    instance_name: String,
    percent: u8,
}

/// One physical panel reachable through DDC/CI. Brightness is exposed as a percentage
/// regardless of the raw range reported by the panel.
#[derive(Debug, Clone)]
struct DdcciPanel {
    monitor_id: MonitorId,
    handle: Arc<PhysicalMonitorHandle>,
    values: DdcciBrightnessValues,
}

impl DdcciPanel {
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

/// PnP device instance id out of a device interface path:
/// `\\?\DISPLAY#BOE0900#4&10fd3ab1&0&UID265988#{e6f07b5f-...}` becomes
/// `DISPLAY\BOE0900\4&10FD3AB1&0&UID265988`.
fn pnp_id_from_device_path(path: &str) -> String {
    let path = path.strip_prefix(r"\\?\").unwrap_or(path);
    let path = path.split_once("#{").map_or(path, |(id, _guid)| id);
    path.replace('#', "\\").to_uppercase()
}

/// PnP device instance id out of a WMI instance name, which is the id plus an `_<index>`
/// suffix: `DISPLAY\BOE0900\4&10fd3ab1&0&UID265988_0` becomes
/// `DISPLAY\BOE0900\4&10FD3AB1&0&UID265988`.
fn pnp_id_from_wmi_instance(instance_name: &str) -> String {
    let id = match instance_name.rsplit_once('_') {
        Some((id, index)) if !index.is_empty() && index.bytes().all(|b| b.is_ascii_digit()) => id,
        _ => instance_name,
    };
    id.to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wmi_instance_matches_device_path() {
        let path =
            r"\\?\DISPLAY#BOE0900#4&10fd3ab1&0&UID265988#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}";
        let instance = r"DISPLAY\BOE0900\4&10fd3ab1&0&UID265988_0";
        assert_eq!(
            pnp_id_from_device_path(path),
            r"DISPLAY\BOE0900\4&10FD3AB1&0&UID265988"
        );
        assert_eq!(
            pnp_id_from_wmi_instance(instance),
            pnp_id_from_device_path(path)
        );
    }

    #[test]
    fn wmi_instance_without_index_is_kept() {
        assert_eq!(
            pnp_id_from_wmi_instance(r"DISPLAY\LGD_05E5\4&abc&0&UID8388688"),
            r"DISPLAY\LGD_05E5\4&ABC&0&UID8388688"
        );
    }
}
