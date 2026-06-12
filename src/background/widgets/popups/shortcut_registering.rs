use std::sync::LazyLock;

use parking_lot::Mutex;
use seelen_core::state::{CssStyles, Dialog, DialogContent};
use slu_ipc::messages::SvcAction;
use uuid::Uuid;

use tauri::{Emitter, Listener};

use crate::{
    app::get_app_handle, cli::ServicePipe, error::Result, log_error,
    widgets::trigger_dialog_backend,
};

pub static REG_SHORTCUT_DATA: LazyLock<Mutex<RegShortcutData>> =
    LazyLock::new(|| Mutex::new(RegShortcutData::default()));

#[derive(Default)]
pub struct RegShortcutData {
    pub dialog_id: Uuid,
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
        *self = Default::default();
    }
}

pub fn set_registering_shortcut(shortcut: Option<Vec<String>>) -> Result<()> {
    let mut reg = REG_SHORTCUT_DATA.lock();
    reg.shortcut = shortcut.clone();

    let Some(mut shortcut) = shortcut else {
        reg.emit_state_to_requester();
        reg.cancel_shortcut_registering();
        return Ok(());
    };

    // the library allows differences between keys, but we simplify things for users
    for key in &mut shortcut {
        if key == "LShift" || key == "RShift" {
            *key = "Shift".to_string();
        }
        if key == "LControl" || key == "RControl" {
            *key = "Ctrl".to_string();
        }
        if key == "LMenu" || key == "RMenu" {
            *key = "Alt".to_string();
        }
        if key == "LWin" || key == "RWin" {
            *key = "Win".to_string();
        }
    }

    // save normalized shortcut
    reg.shortcut = Some(shortcut.clone());

    let is_first_trigger = reg.dialog_id.is_nil();
    if is_first_trigger {
        reg.dialog_id = Uuid::new_v4();

        // listen once for each button action; re-registers after cancel/accept
        let dialog_id = reg.dialog_id;
        get_app_handle().once("user_shortcut_accepted", move |_| {
            log_error!(ServicePipe::request(SvcAction::StopShortcutRegistration));
            let mut reg = REG_SHORTCUT_DATA.lock();
            reg.emit_state_to_requester();
            reg.cancel_shortcut_registering();
        });
        get_app_handle().once("shortcut_register_cancelled", move |_| {
            log_error!(ServicePipe::request(SvcAction::StopShortcutRegistration));
            let mut reg = REG_SHORTCUT_DATA.lock();
            if reg.dialog_id == dialog_id {
                reg.cancel_shortcut_registering();
            }
        });
    }

    trigger_dialog_backend(get_dialog(reg.dialog_id, &shortcut))
}

fn get_dialog(id: Uuid, shortcut: &[String]) -> Dialog {
    let items = if shortcut.is_empty() {
        vec![DialogContent::Text {
            value: t!("shortcut.register.placeholder").to_string(),
            styles: Some(
                CssStyles::new()
                    .add("color", "var(--color-gray-400)")
                    .add("fontSize", "1rem")
                    .add("fontStyle", "italic"),
            ),
        }]
    } else {
        shortcut
            .iter()
            .map(|s| DialogContent::Text {
                value: s.to_string(),
                styles: Some(
                    CssStyles::new()
                        .add("backgroundColor", "var(--slu-std-bg-light-color)")
                        .add("padding", "4px 10px")
                        .add("borderRadius", "4px"),
                ),
            })
            .collect()
    };

    let keys_box = DialogContent::Group {
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
            DialogContent::Button {
                skin: Some("default".to_string()),
                inner: vec![DialogContent::Text {
                    value: t!("cancel").to_string(),
                    styles: None,
                }],
                on_click: "shortcut_register_cancelled".to_string(),
                styles: None,
            },
            DialogContent::Button {
                skin: Some("solid".to_string()),
                inner: vec![DialogContent::Text {
                    value: t!("done").to_string(),
                    styles: None,
                }],
                on_click: "user_shortcut_accepted".to_string(),
                styles: None,
            },
        ]
    };

    Dialog {
        identifier: id,
        width: 360.0,
        height: 180.0,
        title: vec![DialogContent::Text {
            value: t!("shortcut.register.title").to_string(),
            styles: None,
        }],
        content: vec![keys_box],
        footer: actions,
    }
}
