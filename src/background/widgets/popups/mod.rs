pub mod cli;
pub mod handlers;
pub mod shortcut_registering;

use std::{collections::HashMap, sync::LazyLock};

use parking_lot::Mutex;
use seelen_core::{handlers::SeelenEvent, resource::WidgetId, state::SluPopupConfig};
use tauri::{
    utils::{config::WindowEffectsConfig, WindowEffect},
    Emitter, Listener, LogicalSize, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};
use uuid::Uuid;

use crate::{app::get_app_handle, error::Result, utils::WidgetWebviewLabel};

pub static POPUPS_MANAGER: LazyLock<Mutex<PopupsManager>> = LazyLock::new(|| {
    Mutex::new(PopupsManager {
        webviews: HashMap::new(),
        configs: HashMap::new(),
        listeners: HashMap::new(),
    })
});

pub struct PopupsManager {
    webviews: HashMap<Uuid, WebviewWindow>,
    configs: HashMap<Uuid, SluPopupConfig>,
    pub listeners: HashMap<Uuid, Vec<u32>>,
}

impl PopupsManager {
    pub fn get_window_handle(&self, id: &Uuid) -> Option<&WebviewWindow> {
        self.webviews.get(id)
    }

    pub fn create(&mut self, config: SluPopupConfig) -> Result<Uuid> {
        let popup_id = Uuid::new_v4();
        let label = WidgetWebviewLabel::new(&WidgetId::known_popup(), None, Some(&popup_id));

        let manager = get_app_handle();
        let window = WebviewWindowBuilder::new(
            manager,
            label.raw,
            tauri::WebviewUrl::App("popup/index.html".into()),
        )
        .center()
        .minimizable(false)
        .maximizable(false)
        .resizable(false)
        .closable(false)
        .always_on_top(true)
        .decorations(false)
        .transparent(true)
        .inner_size(config.width, config.height)
        .effects(WindowEffectsConfig {
            color: None,
            radius: None,
            state: None,
            effects: vec![WindowEffect::Acrylic],
        })
        .visible(false)
        .build()?;

        window.on_window_event(move |e| {
            if let WindowEvent::Destroyed = e {
                log::trace!("popup destroyed: {popup_id}");
                std::thread::spawn(move || {
                    let mut popups = POPUPS_MANAGER.lock();
                    popups.webviews.remove(&popup_id);
                    popups.configs.remove(&popup_id);
                    if let Some(tokens) = popups.listeners.remove(&popup_id) {
                        for token in tokens {
                            get_app_handle().unlisten(token);
                        }
                    }
                });
            }
        });

        window.center()?; // ensure centered position
        self.configs.insert(popup_id, config);
        self.webviews.insert(popup_id, window);
        Ok(popup_id)
    }

    pub fn update(&mut self, id: &Uuid, config: SluPopupConfig) -> Result<()> {
        if let Some(webview) = self.webviews.get(id) {
            webview.emit(SeelenEvent::PopupContentChanged, &config)?;
            webview.set_size(LogicalSize::new(config.width, config.height))?;
            webview.center()?;
        }
        self.configs.insert(*id, config);
        Ok(())
    }

    pub fn close_popup(&mut self, id: &Uuid) -> Result<()> {
        if let Some(webview) = self.webviews.get(id) {
            webview.close()?;
        } else {
            return Err("popup not found".into());
        }
        Ok(())
    }
}
