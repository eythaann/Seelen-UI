use seelen_core::state::shortcuts::{SluHotkeyAction, SluShortcutsSettings};
use slu_ipc::AppIpc;
use win_hotkeys::{error::WHKError, Hotkey, HotkeyManager, VKey};

use crate::{error::Result, log_error};

pub fn start_app_shortcuts(config: SluShortcutsSettings) -> Result<()> {
    if let Err(err) = HotkeyManager::start_keyboard_capturing() {
        match err {
            WHKError::AlreadyStarted => {}
            others => return Err(others.into()),
        }
    };

    let mut manager = HotkeyManager::current();
    manager.unregister_all()?; // delete previously registered shortcuts

    'registration: for slu_hotkey in &config.app_commands {
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
        let tokio_handle = tokio::runtime::Handle::current();

        let hotkey = Hotkey::from_keys(vkeys).action(move || {
            tokio_handle.spawn(async move {
                println!("action: {action:?}");
                let command = hotkey_action_to_cli_command(action);
                println!("command: {command:?}");
                log_error!(AppIpc::send(command).await);
            });
        });

        log_error!(manager.register_hotkey(hotkey), slu_hotkey);
    }
    Ok(())
}

pub fn stop_app_shortcuts() {
    HotkeyManager::stop_keyboard_capturing();
}

pub fn _request_to_user_input_shortcut() {}

fn hotkey_action_to_cli_command(action: SluHotkeyAction) -> Vec<String> {
    use SluHotkeyAction::*;
    let mut args = Vec::new();
    let cmd = match action {
        // Virtual Desktop
        SwitchToNextWorkspace => vec!["vd", "switch-next"],
        SwitchToPreviousWorkspace => vec!["vd", "switch-prev"],
        SwitchWorkspace(index) => {
            args.push(index.to_string());
            vec!["vd", "switch-workspace", &args[0]]
        }
        MoveToWorkspace(index) => {
            args.push(index.to_string());
            vec!["vd", "move-to-workspace", &args[0]]
        }
        SendToWorkspace(index) => {
            args.push(index.to_string());
            vec!["vd", "send-to-workspace", &args[0]]
        }
        CreateNewWorkspace => vec!["vd", "create-new-workspace"],
        DestroyCurrentWorkspace => vec!["vd", "destroy-current-workspace"],
        // Weg
        StartWegApp(index) => {
            args.push(index.to_string());
            vec!["weg", "foreground-or-run-app", &args[0]]
        }
        // App Launcher / Start Menu
        ToggleLauncher => vec!["launcher", "toggle"],
        // Window Manager
        IncreaseWidth => vec!["wm", "width", "increase"],
        DecreaseWidth => vec!["wm", "width", "decrease"],
        IncreaseHeight => vec!["wm", "height", "increase"],
        DecreaseHeight => vec!["wm", "height", "decrease"],
        RestoreSizes => vec!["wm", "reset-workspace-size"],
        // Window Manger focused window sizing
        FocusTop => vec!["wm", "focus", "top"],
        FocusBottom => vec!["wm", "focus", "bottom"],
        FocusLeft => vec!["wm", "focus", "left"],
        FocusRight => vec!["wm", "focus", "right"],
        // Window Manager focused window positioning
        MoveWindowUp => vec!["wm", "move", "top"],
        MoveWindowDown => vec!["wm", "move", "bottom"],
        MoveWindowLeft => vec!["wm", "move", "left"],
        MoveWindowRight => vec!["wm", "move", "right"],
        // Tiling window manager reservation
        ReserveTop => vec!["wm", "reserve", "top"],
        ReserveBottom => vec!["wm", "reserve", "bottom"],
        ReserveLeft => vec!["wm", "reserve", "left"],
        ReserveRight => vec!["wm", "reserve", "right"],
        ReserveFloat => vec!["wm", "reserve", "float"],
        ReserveStack => vec!["wm", "reserve", "stack"],
        // others
        MiscOpenSettings => vec!["settings"],
        MiscToggleLockTracing => vec!["debug", "toggle-trace-lock"],
        MiscToggleWinEventTracing => vec!["debug", "toggle-win-events"],
    };

    cmd.iter().map(|s| s.to_string()).collect()
}
