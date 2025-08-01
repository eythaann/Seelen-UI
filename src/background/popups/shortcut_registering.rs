use std::sync::LazyLock;

use parking_lot::Mutex;
use seelen_core::state::{CssStyles, SluPopupConfig, SluPopupContent};
use slu_ipc::messages::SvcAction;
use tauri::{Emitter, Listener, WindowEvent};
use uuid::Uuid;

use crate::{
    cli::ServicePipe, error_handler::Result, log_error, popups::POPUPS_MANAGER,
    seelen::get_app_handle,
};

pub static REG_SHORTCUT_DATA: LazyLock<Mutex<RegShortcutData>> =
    LazyLock::new(|| Mutex::new(RegShortcutData::default()));

#[derive(Default)]
pub struct RegShortcutData {
    pub popup_id: Uuid,
    pub shortcut: Option<Vec<String>>,
    pub response_view_label: Option<String>,
    pub response_event: Option<String>,
}

impl RegShortcutData {
    fn emit_state_to_requester(&self) {
        if let (Some(label), Some(event)) = (&self.response_view_label, &self.response_event) {
            log_error!(get_app_handle().emit_to(label, event, &self.shortcut));
        }
    }

    fn cancel_shortcut_registering(&mut self) {
        if !self.popup_id.is_nil() {
            // could fail if was already closed
            let _ = POPUPS_MANAGER.lock().close_popup(&self.popup_id);
        }
        *self = Default::default();
    }
}

pub fn set_registering_shortcut(shortcut: Option<Vec<String>>) -> Result<()> {
    let mut reg = REG_SHORTCUT_DATA.lock();
    reg.shortcut = shortcut.clone();

    let Some(shortcut) = shortcut else {
        reg.emit_state_to_requester();
        reg.cancel_shortcut_registering();
        return Ok(());
    };

    let popup_config = get_popup_config(&shortcut);

    if !reg.popup_id.is_nil() {
        POPUPS_MANAGER.lock().update(&reg.popup_id, popup_config)?;
        return Ok(());
    }

    let mut popup_manager = POPUPS_MANAGER.lock();
    reg.popup_id = popup_manager.create(popup_config)?;

    let handle = popup_manager.get_window_handle(&reg.popup_id).unwrap();

    // stop on close of the popup
    handle.on_window_event(|e| {
        if let WindowEvent::Destroyed = e {
            log_error!(ServicePipe::request(SvcAction::StopShortcutRegistration));
        }
    });

    handle.once("user_shortcut_accepted", |_| {
        let mut reg = REG_SHORTCUT_DATA.lock();
        reg.emit_state_to_requester();
        reg.cancel_shortcut_registering();
    });
    Ok(())
}

fn get_popup_config(shortcut: &[String]) -> SluPopupConfig {
    let items = if shortcut.is_empty() {
        vec![SluPopupContent::Text {
            value: t!("shortcut.register.placeholder").to_string(),
            styles: Some(
                CssStyles::new()
                    .add("color", "#6c6c6c")
                    .add("fontSize", "1rem")
                    .add("fontStyle", "italic"),
            ),
        }]
    } else {
        shortcut
            .iter()
            .map(|s| SluPopupContent::Text {
                value: s.to_string(),
                styles: Some(
                    CssStyles::new()
                        .add("backgroundColor", "#fefefe")
                        .add("padding", "4px 10px")
                        .add("borderRadius", "4px"),
                ),
            })
            .collect()
    };

    let keys_box = SluPopupContent::Group {
        items,
        styles: Some(
            CssStyles::new()
                .add("width", "100%")
                .add("height", "100%")
                .add("display", "flex")
                .add("gap", "5px")
                .add("justifyContent", "center")
                .add("alignItems", "center")
                .add("flexWrap", "wrap")
                .add("fontWeight", "bold"),
        ),
    };

    let actions = if shortcut.is_empty() {
        Vec::new()
    } else {
        vec![
            SluPopupContent::Button {
                inner: vec![SluPopupContent::Text {
                    value: t!("cancel").to_string(),
                    styles: None,
                }],
                on_click: "exit".to_string(),
                styles: Some(
                    CssStyles::new()
                        .add("backgroundColor", "var(--color-red-700)")
                        .add("color", "var(--color-white)"),
                ),
            },
            SluPopupContent::Button {
                inner: vec![SluPopupContent::Text {
                    value: t!("done").to_string(),
                    styles: None,
                }],
                on_click: "user_shortcut_accepted".to_string(),
                styles: None,
            },
        ]
    };

    SluPopupConfig {
        width: 360.0,
        height: 180.0,
        title: vec![SluPopupContent::Text {
            value: t!("shortcut.register.title").to_string(),
            styles: None,
        }],
        content: vec![keys_box],
        footer: actions,
    }
}
