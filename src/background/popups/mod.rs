pub mod handlers;

use std::{collections::HashMap, sync::LazyLock};

use base64::Engine;
use parking_lot::Mutex;
use seelen_core::{
    resource::{Resource, ResourceKind, ResourceText},
    state::{CssStyles, SluPopupConfig, SluPopupContent},
};
use tauri::{
    utils::{config::WindowEffectsConfig, WindowEffect},
    Listener, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};
use uuid::Uuid;

use crate::{
    error_handler::Result, log_error, seelen::get_app_handle, state::application::FULL_STATE,
};

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
    listeners: HashMap<Uuid, Vec<u32>>,
}

impl PopupsManager {
    pub fn create(&mut self, config: SluPopupConfig) -> Result<Uuid> {
        let popup_id = Uuid::new_v4();
        let label = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(format!("@seelen/popup?instanceId={popup_id}"));

        let manager = get_app_handle();
        let window = WebviewWindowBuilder::new(
            manager,
            label,
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
        .inner_size(450.0, 240.0)
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
                log::trace!("popup destroyed: {}", popup_id);
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

        self.configs.insert(popup_id, config);
        self.webviews.insert(popup_id, window);
        Ok(popup_id)
    }

    pub fn close_popup(&mut self, id: Uuid) -> Result<()> {
        if let Some(webview) = self.webviews.get(&id) {
            webview.close()?;
        } else {
            return Err("popup not found".into());
        }
        Ok(())
    }

    pub fn create_added_resource(&mut self, resource: &Resource) -> Result<()> {
        let handle = get_app_handle();
        let config = resource_to_popup_config(resource)?;
        let popup_id = self.create(config)?;

        let id = resource.id;
        let kind = resource.kind.clone();
        let event = format!("resource::{id}::enable");
        let token = handle.once(event, move |_e| {
            std::thread::spawn(move || {
                println!("add_to_active");
                FULL_STATE.rcu(move |state| {
                    let mut state = state.cloned();
                    match kind {
                        ResourceKind::Theme => {
                            state.settings.selected_themes.push(format!("{id}.slu"));
                        }
                        ResourceKind::IconPack => {
                            state.settings.icon_packs.push(format!("{id}.slu"));
                        }
                        _ => {}
                    }
                    state
                });
                log_error!(FULL_STATE.load().write_settings());
                log_error!(POPUPS_MANAGER.lock().close_popup(popup_id));
            });
        });

        self.listeners.entry(resource.id).or_default().push(token);
        Ok(())
    }
}

fn resource_to_popup_config(resource: &Resource) -> Result<SluPopupConfig> {
    let mut popup = SluPopupConfig::default();

    popup.title.push(SluPopupContent::Group {
        items: vec![
            SluPopupContent::Icon {
                name: "GrCircleInformation".to_string(),
                styles: None,
            },
            SluPopupContent::Text {
                value: t!("resource.added").to_string(),
                styles: None,
            },
        ],
        styles: Some(CssStyles::new().add("alignItems", "center")),
    });

    let image_styles = CssStyles::new()
        .add("width", "90px")
        .add("minWidth", "90px")
        .add("height", "90px")
        .add("borderRadius", "14px")
        .add("backgroundColor", "var(--color-gray-200)")
        .add("display", "flex")
        .add("alignItems", "center")
        .add("justifyContent", "center");

    let image = if let Some(url) = &resource.metadata.portrait {
        SluPopupContent::Image {
            href: url.clone(),
            styles: Some(image_styles),
        }
    } else {
        SluPopupContent::Icon {
            name: "GrStatusUnknown".to_string(),
            styles: Some(image_styles),
        }
    };

    popup.content = vec![SluPopupContent::Group {
        items: vec![
            image,
            SluPopupContent::Group {
                items: vec![
                    SluPopupContent::Text {
                        value: resource_text_to_string(&resource.metadata.display_name),
                        styles: Some(
                            CssStyles::new()
                                .add("fontWeight", "bold")
                                .add("fontSize", "2rem")
                                .add("lineHeight", "1.2em"),
                        ),
                    },
                    SluPopupContent::Text {
                        value: resource_text_to_string(&resource.metadata.description),
                        styles: None,
                    },
                ],
                styles: Some(CssStyles::new().add("flexDirection", "column")),
            },
        ],
        styles: None,
    }];

    popup.footer = vec![SluPopupContent::Button {
        inner: vec![SluPopupContent::Text {
            value: t!("resource.enable").to_string(),
            styles: None,
        }],
        on_click: format!("resource::{}::enable", resource.id),
        styles: None,
    }];

    Ok(popup)
}

fn resource_text_to_string(text: &ResourceText) -> String {
    match text {
        ResourceText::En(s) => s.to_string(),
        ResourceText::Localized(m) => {
            let locale = FULL_STATE.load().locale();
            m.get(&locale)
                .or_else(|| m.get("en"))
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Unknown".to_owned())
        }
    }
}
