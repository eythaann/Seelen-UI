use seelen_core::state::shortcuts::ResolvedShortcut;
use slu_ipc::{messages::AppMessage, AppIpc};
use win_hotkeys::{
    error::WHKError, events::KeyboardInputEvent, Hotkey, HotkeyManager, TriggerTiming, VKey,
};

use crate::{
    app_management::kill_all_seelen_ui_processes, error::Result, exit, get_async_handler, log_error,
};

pub fn apply_shortcuts(shortcuts: Vec<ResolvedShortcut>) -> Result<()> {
    if let Err(err) = HotkeyManager::start_keyboard_capturing() {
        match err {
            WHKError::AlreadyStarted => {}
            others => return Err(others.into()),
        }
    };

    let manager = HotkeyManager::current();
    manager.unregister_all()?;

    if shortcuts.is_empty() {
        return Ok(());
    }

    'registration: for s in shortcuts {
        if s.keys.is_empty() {
            continue 'registration;
        }

        let mut vkeys = Vec::new();
        for key in &s.keys {
            let vkey = match VKey::from_keyname(key) {
                Ok(vkey) => vkey,
                Err(e) => {
                    log::warn!("Failed to parse shortcut {:?} error: {e}", s.keys);
                    continue 'registration;
                }
            };
            vkeys.push(vkey);
        }

        let command = s.command.clone();
        let mut hotkey = Hotkey::from_keys(&vkeys).action(move || {
            log::trace!("Hotkey triggered: {command:?}");
            match command.as_slice() {
                [a, b] if a == "service" && b == "force-restart" => {
                    log_error!(kill_all_seelen_ui_processes());
                }
                [a, b] if a == "service" && b == "force-quit" => {
                    crate::EXITING.store(true, std::sync::atomic::Ordering::SeqCst);
                    log_error!(kill_all_seelen_ui_processes());
                    exit(0);
                }
                _ => {
                    let cmd = command.clone();
                    get_async_handler().spawn(async move {
                        log_error!(AppIpc::send(AppMessage::Cli(cmd)).await);
                    });
                }
            }
        });

        if vkeys.len() == 1 {
            hotkey.trigger_timing = TriggerTiming::OnKeyUp;
            hotkey.strict_sequence = true;
        }

        log_error!(manager.register_hotkey(hotkey));
    }
    Ok(())
}

pub fn stop_app_shortcuts() {
    HotkeyManager::stop_keyboard_capturing();
}

pub async fn start_shortcut_registration() -> Result<()> {
    let hkm = HotkeyManager::current();

    let handle = tokio::runtime::Handle::current();
    let on_free_keyboard = move || {
        handle.spawn(async {
            let _ = send_registering_to_app(None).await;
        });
        HotkeyManager::current().remove_global_keyboard_listener();
    };

    let handle = tokio::runtime::Handle::current();
    let on_keyboard_event = move |event| {
        handle.spawn(async {
            match event {
                KeyboardInputEvent::KeyDown { key, state } => {
                    if key == VKey::Escape {
                        return;
                    }
                    let keys = state.pressing.iter().map(|vkey| vkey.to_string()).collect();
                    let _ = send_registering_to_app(Some(keys)).await;
                }
                KeyboardInputEvent::KeyUp { .. } => {}
            }
        });
    };

    send_registering_to_app(Some(vec![])).await?;
    hkm.steal_keyboard(on_free_keyboard);
    hkm.set_global_keyboard_listener(on_keyboard_event);
    Ok(())
}

pub async fn stop_shortcut_registration() -> Result<()> {
    HotkeyManager::current().free_keyboard();
    Ok(())
}

async fn send_registering_to_app(hotkey: Option<Vec<String>>) -> Result<()> {
    AppIpc::send(AppMessage::Cli(vec![
        "popup".to_owned(),
        "internal-set-shortcut".to_owned(),
        serde_json::to_string(&hotkey)?,
    ]))
    .await?;
    Ok(())
}
