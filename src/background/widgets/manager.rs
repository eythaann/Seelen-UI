use std::{
    sync::{
        LazyLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::sleep,
    time::Duration,
};

use seelen_core::{
    resource::WidgetId,
    state::{WidgetLoader, WidgetStatus},
    system_state::PowerMode,
};

use crate::{
    error::{Result, ResultLogExt},
    resources::RESOURCES,
    state::FULL_STATE,
    system_modules::{
        monitors::MonitorManager,
        power::{PowerEvent, PowerManager},
    },
    utils::lock_free::SyncHashMap,
    widgets::{WidgetWebviewLabel, loader::WidgetDeployment},
};

pub static WIDGET_MANAGER: LazyLock<WidgetManager> = LazyLock::new(WidgetManager::create);
pub static GAME_MODE_ACTIVE: AtomicBool = AtomicBool::new(false);
static PREWARM_GENERATION: AtomicU64 = AtomicU64::new(0);

const BUILT_IN_TASKBAR_POPUPS: &[(&str, &str)] = &[
    ("@seelen/weg", "@seelen/weg-preview"),
    ("@seelen/fancy-toolbar", "@seelen/system-tray"),
    ("@seelen/fancy-toolbar", "@seelen/calendar-popup"),
    ("@seelen/fancy-toolbar", "@seelen/bluetooth-popup"),
    ("@seelen/fancy-toolbar", "@seelen/network-popup"),
    ("@seelen/fancy-toolbar", "@seelen/media-popup"),
    ("@seelen/fancy-toolbar", "@seelen/notifications"),
    ("@seelen/fancy-toolbar", "@seelen/quick-settings"),
];
const PREWARM_STAGGER: Duration = Duration::from_millis(500);

pub struct WidgetManager {
    /// group of widgets instances by widget resource id
    pub deployments: SyncHashMap<WidgetId, WidgetDeployment>,
}

impl WidgetManager {
    fn create() -> Self {
        let sub_id = MonitorManager::subscribe(|_event| {
            WIDGET_MANAGER.reconcile().log_error();
        });
        MonitorManager::set_event_handler_priority(&sub_id, 1);

        PowerManager::subscribe(|event| {
            if let PowerEvent::ModeChanged { mode } = event
                && FULL_STATE.load().settings.suspend_on_game_mode
            {
                match mode {
                    PowerMode::GameMode => WIDGET_MANAGER.suspend_all(),
                    _ => WIDGET_MANAGER.resume_all().log_error(),
                }
            }
        });
        // events are only emitted once the manager exists
        PowerManager::instance();
        let suspend_on_game_mode = FULL_STATE.load().settings.suspend_on_game_mode;
        GAME_MODE_ACTIVE.store(
            suspend_on_game_mode && PowerManager::instance().mode() == PowerMode::GameMode,
            Ordering::Release,
        );

        Self {
            deployments: SyncHashMap::new(),
        }
    }

    pub fn is_ready(&self, label: &WidgetWebviewLabel) -> bool {
        self.deployments
            .get(&label.widget_id, |deploy| {
                deploy.pods.any(|(key, pod)| key == label && pod.is_ready())
            })
            .unwrap_or(false)
    }

    pub fn set_status(&self, label: &WidgetWebviewLabel, status: WidgetStatus) {
        self.deployments.get(&label.widget_id, |deploy| {
            deploy.pods.get(label, |instance| {
                instance.set_status(status);
            });
        });
    }

    pub fn suspend_all(&self) {
        PREWARM_GENERATION.fetch_add(1, Ordering::AcqRel);
        GAME_MODE_ACTIVE.store(true, Ordering::Release);
        self.deployments.for_each(|(_, deploy)| {
            deploy.pods.clear();
        });
    }

    pub fn resume_all(&self) -> Result<()> {
        GAME_MODE_ACTIVE.store(false, Ordering::Release);
        self.reconcile()
    }

    pub fn reconcile(&self) -> Result<()> {
        PREWARM_GENERATION.fetch_add(1, Ordering::AcqRel);
        let ready_host = [WidgetId::known_toolbar(), WidgetId::known_weg()]
            .into_iter()
            .find(|host| {
                self.deployments
                    .get(host, |deployment| {
                        deployment.pods.any(|(_, pod)| pod.is_ready())
                    })
                    .unwrap_or(false)
            });

        // remove deleted resources
        self.deployments
            .retain(|(key, _)| RESOURCES.widgets.contains_sync(key));

        let mut filtered = Vec::new();
        RESOURCES.widgets.iter_sync(|k, w| {
            if w.loader != WidgetLoader::Legacy {
                filtered.push((k.clone(), w.clone()));
            }
            true
        });

        let state = FULL_STATE.load();
        for (id, widget) in filtered {
            if !state.is_widget_enabled(&id) {
                self.deployments.remove(&id);
                continue;
            }

            if !self.deployments.contains_key(&id) {
                self.deployments
                    .upsert(id.clone(), WidgetDeployment::new(widget));
            }
        }

        // lazy creation of webviews to reduce startup time
        std::thread::spawn(move || {
            fn reconcile(deployment: &WidgetDeployment) {
                deployment.reconcile();
                if !deployment.definition.lazy && !GAME_MODE_ACTIVE.load(Ordering::Acquire) {
                    deployment.start_all_webviews();
                }
            }

            // More visual widgets load first
            for priority in [
                WidgetId::known_wall(),
                WidgetId::known_toolbar(),
                WidgetId::known_weg(),
            ] {
                WIDGET_MANAGER.deployments.get(&priority, |deployment| {
                    reconcile(deployment);
                });
            }

            // All other widgets
            WIDGET_MANAGER.deployments.for_each(|(_, deployment)| {
                reconcile(deployment);
            });

            if let Some(host) = ready_host {
                schedule_taskbar_popup_prewarm(&host);
            }
        });

        Ok(())
    }
}

pub fn schedule_taskbar_popup_prewarm(host: &WidgetId) {
    if *host != WidgetId::known_toolbar() && *host != WidgetId::known_weg() {
        return;
    }

    let generation = PREWARM_GENERATION.fetch_add(1, Ordering::AcqRel) + 1;
    std::thread::spawn(move || {
        for (position, (popup_host, popup)) in BUILT_IN_TASKBAR_POPUPS.iter().enumerate() {
            if position > 0 {
                sleep(PREWARM_STAGGER);
            }
            if generation != PREWARM_GENERATION.load(Ordering::Acquire)
                || GAME_MODE_ACTIVE.load(Ordering::Acquire)
                || !crate::platform::session::is_interactive()
            {
                return;
            }

            let host_id = WidgetId::from(*popup_host);
            if !WIDGET_MANAGER
                .deployments
                .get(&host_id, |deployment| {
                    deployment.pods.any(|(_, pod)| pod.is_ready())
                })
                .unwrap_or(false)
            {
                continue;
            }

            WIDGET_MANAGER
                .deployments
                .get(&WidgetId::from(*popup), |deployment| {
                    deployment.start_all_webviews();
                });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prewarm_only_targets_builtin_taskbar_popups() {
        assert_eq!(BUILT_IN_TASKBAR_POPUPS.len(), 8);
        assert!(BUILT_IN_TASKBAR_POPUPS.iter().all(|(host, popup)| {
            (*host == "@seelen/fancy-toolbar" || *host == "@seelen/weg")
                && popup.starts_with("@seelen/")
        }));
    }
}
