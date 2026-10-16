use std::sync::{
    LazyLock,
    atomic::{AtomicBool, Ordering},
};

use seelen_core::state::{CssStyles, Dialog, DialogContent};
use slu_ipc::messages::SvcAction;
use tokio::sync::oneshot;
use uuid::Uuid;

use tauri::{EventId, Listener};

use crate::{
    app::get_app_handle,
    cli::ServicePipe,
    error::{Result, ResultLogExt},
    utils::lock_free::TracedMutex,
    widgets::trigger_dialog_backend,
};

/// Whether a shortcut registering is in progress. There is only one keyboard to steal,
/// so only one registering can run at a time, no matter who asked for it.
static REGISTERING: AtomicBool = AtomicBool::new(false);

/// State of the in-progress shortcut registering (empty when idle). The lock is only taken
/// to read or update this state, never while showing the dialog or calling the service,
/// since those wait on other threads and on the user.
static REG_SHORTCUT_DATA: LazyLock<TracedMutex<RegShortcutData>> =
    LazyLock::new(|| TracedMutex::new(RegShortcutData::default()));

#[derive(Default)]
struct RegShortcutData {
    /// Nil until the dialog is shown for the first time.
    dialog_id: Uuid,
    shortcut: Option<Vec<String>>,
    /// Resolves the pending `request_to_user_input_shortcut` call.
    responder: Option<oneshot::Sender<Option<Vec<String>>>>,
    /// Dialog button listeners, removed when the registering finishes.
    listeners: Vec<EventId>,
}

/// How a registering ends, decides the result.
enum Outcome {
    /// The registered shortcut, `None` if no keys were pressed.
    Accepted,
    /// Always `None`.
    Cancelled,
}

/// Starts a registering, the receiver gets its result: the shortcut, or `None` if it was
/// cancelled or no keys were pressed. Returns `None` (and warns) if one is already in progress.
pub fn begin_shortcut_registering() -> Option<oneshot::Receiver<Option<Vec<String>>>> {
    if REGISTERING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        log::warn!("Ignoring shortcut registering request: one is already in progress");
        return None;
    }
    let (tx, rx) = oneshot::channel();
    REG_SHORTCUT_DATA.lock().responder = Some(tx);
    Some(rx)
}

/// Ends the registering without a result, its receiver is dropped (e.g. the service
/// failed to start it).
pub fn abort_shortcut_registering() {
    let reg = std::mem::take(&mut *REG_SHORTCUT_DATA.lock());
    REGISTERING.store(false, Ordering::Release);
    remove_listeners(reg.listeners);
}

/// Called by the service with the keys being pressed, or `None` when it released the
/// keyboard (Escape or stopped).
pub fn set_registering_shortcut(shortcut: Option<Vec<String>>) -> Result<()> {
    if !REGISTERING.load(Ordering::Acquire) {
        log::warn!("Ignoring shortcut update: no shortcut registering in progress");
        return Ok(());
    }

    let Some(mut shortcut) = shortcut else {
        finish_registering(None, Outcome::Cancelled);
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

    let (dialog_id, is_first_trigger) = {
        let mut reg = REG_SHORTCUT_DATA.lock();
        // save normalized shortcut
        reg.shortcut = Some(shortcut.clone());
        let is_first_trigger = reg.dialog_id.is_nil();
        if is_first_trigger {
            reg.dialog_id = Uuid::new_v4();
        }
        (reg.dialog_id, is_first_trigger)
    };

    if is_first_trigger {
        // the state is finished first so the result is delivered before the service
        // releases the keyboard (which reports `None` and would cancel it)
        let app = get_app_handle();
        let listeners = vec![
            app.once("user_shortcut_accepted", move |_| {
                finish_registering(Some(dialog_id), Outcome::Accepted);
                ServicePipe::request(SvcAction::StopShortcutRegistration).log_error();
            }),
            app.once("shortcut_register_cancelled", move |_| {
                finish_registering(Some(dialog_id), Outcome::Cancelled);
                ServicePipe::request(SvcAction::StopShortcutRegistration).log_error();
            }),
        ];

        let mut reg = REG_SHORTCUT_DATA.lock();
        if reg.dialog_id == dialog_id {
            reg.listeners.extend(listeners);
        } else {
            // finished meanwhile
            drop(reg);
            remove_listeners(listeners);
        }
    }

    trigger_dialog_backend(get_dialog(dialog_id, &shortcut))
}

/// Resets the state and resolves the request. With `dialog_id`, only finishes if it is
/// still the current registering (a stale dialog button must not end a newer one).
fn finish_registering(dialog_id: Option<Uuid>, outcome: Outcome) {
    let reg = {
        let mut reg = REG_SHORTCUT_DATA.lock();
        if dialog_id.is_some_and(|id| id != reg.dialog_id) {
            return;
        }
        let reg = std::mem::take(&mut *reg);
        // still under the lock: a new registering can't fill the state before it is taken
        REGISTERING.store(false, Ordering::Release);
        reg
    };

    remove_listeners(reg.listeners);
    if let Some(responder) = reg.responder {
        let result = match outcome {
            Outcome::Accepted => reg.shortcut.filter(|keys| !keys.is_empty()),
            Outcome::Cancelled => None,
        };
        // the request may have been dropped (webview closed), nothing to do then
        let _ = responder.send(result);
    }
}

fn remove_listeners(listeners: Vec<EventId>) {
    let app = get_app_handle();
    for id in listeners {
        app.unlisten(id);
    }
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
