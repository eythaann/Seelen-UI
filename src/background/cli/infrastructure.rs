use win_hotkeys::{Hotkey, HotkeyManager, VKey};

use crate::{
    cli::hotkeys::hotkey_action_to_cli_command, error_handler::Result, log_error,
    state::application::FULL_STATE,
};

pub fn start_app_shortcuts() -> Result<()> {
    let _ = HotkeyManager::start_keyboard_capturing(); // will fail if already started

    let mut manager = HotkeyManager::current();
    manager.unregister_all()?;

    let state = FULL_STATE.load();
    'registration: for slu_hotkey in &state.settings.shortcuts.app_commands {
        let mut vkeys = Vec::new();
        for key in &slu_hotkey.keys {
            let vkey = match VKey::from_keyname(key) {
                Ok(vkey) => vkey,
                Err(e) => {
                    log::warn!("Failed to parse shortcut {:?} error: {e}", slu_hotkey.keys);
                    continue 'registration;
                }
            };
            vkeys.push(vkey);
        }

        let action = slu_hotkey.action;
        let hotkey = Hotkey::from_keys(vkeys).action(move || {
            println!("action: {action:?}");
            let command = hotkey_action_to_cli_command(action);
            log_error!(command.process());
        });

        log_error!(manager.register_hotkey(hotkey), slu_hotkey);
    }

    Ok(())
}

pub fn stop_app_shortcuts() {
    HotkeyManager::stop_keyboard_capturing();
}

#[tauri::command(async)]
pub fn request_to_user_input_shortcut() {}
