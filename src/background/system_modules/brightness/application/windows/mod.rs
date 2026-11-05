mod ddcci;
mod wmi_types;

use std::{sync::Arc, time::Duration};

use seelen_core::system_state::MonitorId;
use wmi::WMIConnection;

use crate::{
    error::Result,
    platform::monitor::{Monitor, WindowsMonitorExt},
    system_modules::brightness::domain::{BrightnessEvent, BrightnessSystem},
    utils::lock_free::{SyncVec, TracedMutex},
};

use self::{
    ddcci::{DdcciBrightnessValues, PhysicalMonitorHandle},
    wmi_types::{
        WmiMonitorBrightness, WmiMonitorBrightnessEvent, WmiMonitorBrightnessMethods,
        WmiSetBrightnessPayload,
    },
};

/// `WBEM_E_NOT_SUPPORTED`: the provider has no instances to offer on this hardware.
const WBEM_E_NOT_SUPPORTED: i32 = 0x8004100C_u32 as i32;

/// DDC/CI has no change notifications, so external monitors are polled at this interval to
/// pick up changes made from the monitor's own OSD. Each read costs one I2C round trip
/// (~60 ms on a healthy monitor) on a background thread.
const DDCCI_POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Serializes every DDC/CI transaction (polling and writes) *together with* the publication
/// of its result into `Panels::ddcci`. Holding the lock across both steps means a
/// read that started before a write can never overwrite the written value afterwards.
/// Concurrent transactions over the same I2C bus can also corrupt each other.
static DDCCI_LOCK: TracedMutex<()> = TracedMutex::new(());

type EventCallback = Box<dyn Fn(BrightnessEvent) + Send + Sync>;

/// Tracks the brightness of every panel, keyed by the stable id of the monitor showing it.
/// A monitor usually has a single panel, but a cloned/mirrored group has one per display.
pub struct WindowsBrightness {
    panels: Option<Arc<Panels>>,
    /// Requests to the background worker (owned by it while the panels exist).
    worker_tx: Option<crossbeam_channel::Sender<WorkerMessage>>,
}

impl BrightnessSystem for WindowsBrightness {
    fn create() -> Self {
        Self {
            panels: None,
            worker_tx: None,
        }
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(BrightnessEvent) + Send + Sync + 'static,
    {
        let panels = Arc::new(Panels {
            wmi: SyncVec::new(),
            ddcci: SyncVec::new(),
            on_event: Box::new(on_event),
        });
        self.panels = Some(panels.clone());

        let (worker_tx, worker_rx) = crossbeam_channel::unbounded();
        let worker_panels = panels.clone();
        std::thread::spawn(move || worker_panels.worker(worker_rx));
        let _ = worker_tx.send(WorkerMessage::Rescan);
        self.worker_tx = Some(worker_tx.clone());

        panels.listen_wmi(worker_tx)
    }

    fn rescan(&self) {
        if let Some(tx) = &self.worker_tx {
            let _ = tx.send(WorkerMessage::Rescan);
        }
    }

    fn brightness(&self, id: &MonitorId) -> Option<u8> {
        let panels = self.panels.as_ref()?;
        panels
            .wmi
            .find_and_clone(|panel| &panel.monitor_id == id)
            .map(|panel| panel.percent)
            .or_else(|| {
                panels
                    .ddcci
                    .find_and_clone(|panel| &panel.monitor_id == id)
                    .map(|panel| panel.percent())
            })
    }

    fn set_brightness(&self, id: &MonitorId, percent: u8) -> Result<()> {
        let panels = self.panels.as_ref().ok_or("Brightness not initialized")?;

        let mut supported = false;
        for panel in panels.wmi.to_vec() {
            if &panel.monitor_id == id {
                supported = true;
                // the cached value is updated by the WMI notification
                wmi_set(&panel.instance_name, percent)?;
            }
        }
        supported |= panels.ddcci_set(id, percent)?;

        if !supported {
            return Err("Monitor does not support brightness control".into());
        }
        Ok(())
    }
}

impl Drop for WindowsBrightness {
    fn drop(&mut self) {
        if let Some(tx) = self.worker_tx.take() {
            let _ = tx.send(WorkerMessage::Stop);
        }
    }
}

enum WorkerMessage {
    /// A display was added/removed, re-enumerate monitors.
    Rescan,
    Stop,
}

/// Known panels, shared with the background worker and the WMI listener.
struct Panels {
    /// Internal displays (laptops), managed through WMI.
    wmi: SyncVec<WmiPanel>,
    /// External displays, managed through DDC/CI. Written only while holding `DDCCI_LOCK`.
    ddcci: SyncVec<DdcciPanel>,
    on_event: EventCallback,
}

impl Panels {
    fn emit_changed(&self) {
        (self.on_event)(BrightnessEvent::Changed);
    }

    /// WMI does notify brightness changes (keyboard keys, Windows settings, power plans).
    fn listen_wmi(
        self: &Arc<Self>,
        worker_tx: crossbeam_channel::Sender<WorkerMessage>,
    ) -> Result<()> {
        let wmi = WMIConnection::with_namespace_path("ROOT\\WMI")?;
        if wmi_query(&wmi)?.is_none() {
            log::debug!("Monitor brightness is not supported through WMI on this system");
            return Ok(());
        }

        let panels = self.clone();
        std::thread::spawn(move || {
            let wmi = WMIConnection::with_namespace_path("ROOT\\WMI")?;
            for event in wmi.notification::<WmiMonitorBrightnessEvent>()? {
                let Ok(event) = event else {
                    continue;
                };
                if !panels.on_wmi_event(event) {
                    let _ = worker_tx.send(WorkerMessage::Rescan);
                }
            }
            Result::Ok(())
        });
        Ok(())
    }

    /// Returns false if the event is about an unknown panel.
    fn on_wmi_event(&self, event: WmiMonitorBrightnessEvent) -> bool {
        let mut known = false;
        let mut changed = false;
        self.wmi.for_each(|panel| {
            if panel.instance_name == event.instance_name {
                known = true;
                changed |= panel.percent != event.brightness;
                panel.percent = event.brightness;
            }
        });
        if changed {
            self.emit_changed();
        }
        known
    }

    /// Background loop: pairs the panels with the monitors on demand and polls the DDC/CI ones.
    fn worker(&self, rx: crossbeam_channel::Receiver<WorkerMessage>) {
        loop {
            match rx.recv_timeout(DDCCI_POLL_INTERVAL) {
                Ok(WorkerMessage::Rescan) => {
                    // collapse bursts of display events into a single scan
                    let mut stop = false;
                    while let Ok(message) = rx.try_recv() {
                        stop |= matches!(message, WorkerMessage::Stop);
                    }
                    if stop {
                        return;
                    }
                    self.scan();
                }
                Ok(WorkerMessage::Stop) => return,
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => self.ddcci_poll(),
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => return,
            }
        }
    }

    /// Enumerates the connected monitors and looks for the panels behind each of them.
    fn scan(&self) {
        let monitors = match Monitor::enumerate() {
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

        let wmi_changed = self.wmi_scan(&targets);
        let ddcci_changed = self.ddcci_scan(&targets);
        if wmi_changed || ddcci_changed {
            self.emit_changed();
        }
    }

    /// Returns whether the known WMI panels changed.
    fn wmi_scan(&self, targets: &[ScanTarget]) -> bool {
        let found = match wmi_read(targets) {
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

    /// Returns whether the known DDC/CI panels changed.
    fn ddcci_scan(&self, targets: &[ScanTarget]) -> bool {
        let _guard = DDCCI_LOCK.lock();
        let mut found = Vec::new();
        for target in targets {
            let ScanTarget { monitor, id, name } = target;
            let Ok(panels) = ddcci::physical_monitors(monitor) else {
                continue;
            };
            for handle in panels {
                let description = handle.description();
                match ddcci_read(&handle) {
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
    /// handled by the rescan requested on monitor changes.
    fn ddcci_poll(&self) {
        let guard = DDCCI_LOCK.lock();
        let mut changed = false;
        for known in self.ddcci.to_vec() {
            let Ok(values) = ddcci_read(&known.handle) else {
                continue;
            };
            if values != known.values {
                changed = true;
                self.ddcci.for_each(|panel| {
                    if Arc::ptr_eq(&panel.handle, &known.handle) {
                        panel.values = values;
                    }
                });
            }
        }
        drop(guard);

        if changed {
            self.emit_changed();
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
    let instances = wmi_query(&wmi)?.unwrap_or_default();
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
