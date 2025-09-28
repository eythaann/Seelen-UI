use seelen_core::state::shortcuts::{SluHotkeyAction, SluShortcutsSettings};
use slu_ipc::{messages::AppMessage, AppIpc};
use win_hotkeys::{error::WHKError, events::KeyboardInputEvent, Hotkey, HotkeyManager, VKey};

use crate::{app_management::kill_seelen_ui_processes, error::Result, log_error};

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
            log::trace!("Hotkey triggered: {action:?}");
            if action == SluHotkeyAction::MiscForceRestart {
                log_error!(kill_seelen_ui_processes());
            }

            if let Some(command) = hotkey_action_to_cli_command(action) {
                tokio_handle.spawn(async move {
                    log_error!(AppIpc::send(AppMessage(command)).await);
                });
            }
        });

        log_error!(manager.register_hotkey(hotkey), slu_hotkey);
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
                KeyboardInputEvent::KeyDown { vk_code, state } => {
                    if VKey::from_vk_code(vk_code) == VKey::Escape {
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
    AppIpc::send(AppMessage(vec![
        "popup".to_owned(),
        "internal-set-shortcut".to_owned(),
        serde_json::to_string(&hotkey)?,
    ]))
    .await?;
    Ok(())
}

fn hotkey_action_to_cli_command(action: SluHotkeyAction) -> Option<Vec<String>> {
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
        // wallpaper manager
        CycleWallpaperNext => vec!["wallpaper", "next"],
        CycleWallpaperPrev => vec!["wallpaper", "prev"],
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
        FocusTop => vec!["wm", "focus", "up"],
        FocusBottom => vec!["wm", "focus", "down"],
        FocusLeft => vec!["wm", "focus", "left"],
        FocusRight => vec!["wm", "focus", "right"],
        // Window Manager focused window positioning
        MoveWindowUp => vec!["wm", "move", "up"],
        MoveWindowDown => vec!["wm", "move", "down"],
        MoveWindowLeft => vec!["wm", "move", "left"],
        MoveWindowRight => vec!["wm", "move", "right"],
        // Tiling window manager reservation
        ReserveTop => vec!["wm", "reserve", "top"],
        ReserveBottom => vec!["wm", "reserve", "bottom"],
        ReserveLeft => vec!["wm", "reserve", "left"],
        ReserveRight => vec!["wm", "reserve", "right"],
        ReserveFloat => vec!["wm", "reserve", "float"],
        ReserveStack => vec!["wm", "reserve", "stack"],
        // Tiling window manager state
        PauseTiling => vec!["wm", "toggle"],
        ToggleMonocle => vec!["wm", "toggle-monocle"],
        ToggleFloat => vec!["wm", "toggle-float"],
        CycleStackNext => vec!["wm", "cycle-stack", "next"],
        CycleStackPrev => vec!["wm", "cycle-stack", "prev"],
        // others
        MiscOpenSettings => vec!["settings"],
        MiscToggleLockTracing => vec!["debug", "toggle-trace-lock"],
        MiscToggleWinEventTracing => vec!["debug", "toggle-win-events"],
        _ => vec![],
    };

    match cmd.is_empty() {
        true => None,
        false => Some(cmd.iter().map(|s| s.to_string()).collect()),
    }
}
