use seelen_core::state::{CssStyles, Dialog, DialogContent};

use tauri::Listener;

use crate::{
    app::get_app_handle,
    error::Result,
    log_error,
    widgets::{show_settings_at, trigger_dialog_backend},
};

pub fn show_shortcut_conflict_popup() -> Result<()> {
    let dialog = get_dialog();
    let event = "open_settings_shortcuts";

    get_app_handle().once(event, move |_| {
        log_error!(show_settings_at("/shortcuts"));
    });

    trigger_dialog_backend(dialog)
}

fn get_dialog() -> Dialog {
    Dialog {
        width: 500.0,
        height: 200.0,
        title: vec![DialogContent::Text {
            value: t!("shortcut.conflicts.title").to_string(),
            styles: None,
        }],
        content: vec![DialogContent::Text {
            value: t!("shortcut.conflicts.body").to_string(),
            styles: Some(CssStyles::new().add("textAlign", "center")),
        }],
        footer: vec![
            DialogContent::Button {
                skin: Some("default".to_string()),
                inner: vec![DialogContent::Text {
                    value: t!("shortcut.conflicts.dismiss").to_string(),
                    styles: None,
                }],
                on_click: "exit".to_string(),
                styles: None,
            },
            DialogContent::Button {
                skin: Some("solid".to_string()),
                inner: vec![DialogContent::Text {
                    value: t!("shortcut.conflicts.review").to_string(),
                    styles: None,
                }],
                on_click: "open_settings_shortcuts".to_string(),
                styles: None,
            },
        ],
        ..Default::default()
    }
}
