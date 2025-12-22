use seelen_core::state::{shortcuts::SluHotkeyAction, Settings};
use slu_ipc::{messages::AppMessage, AppIpc};
use win_hotkeys::{error::WHKError, events::KeyboardInputEvent, Hotkey, HotkeyManager, VKey};

use crate::{app_management::kill_seelen_ui_processes, error::Result, exit, log_error};

pub fn start_app_shortcuts(settings: &Settings) -> Result<()> {
    if let Err(err) = HotkeyManager::start_keyboard_capturing() {
        match err {
            WHKError::AlreadyStarted => {}
            others => return Err(others.into()),
        }
    };

    let mut manager = HotkeyManager::current();
    manager.unregister_all()?; // delete previously registered shortcuts

    'registration: for slu_hotkey in &settings.shortcuts.app_commands {
        if let Some(attached) = &slu_hotkey.attached_to {
            if !settings.is_widget_enabled(attached) {
                continue 'registration;
            }
        }

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
            match action {
                SluHotkeyAction::MiscForceRestart => {
                    log_error!(kill_seelen_ui_processes());
                }
                SluHotkeyAction::MiscForceQuit => {
                    crate::EXITING.store(true, std::sync::atomic::Ordering::SeqCst);
                    log_error!(kill_seelen_ui_processes());
                    exit(0);
                }
                _ => {}
            }

            if let Some(command) = hotkey_action_to_cli_command(action) {
                tokio_handle.spawn(async move {
                    log_error!(AppIpc::send(AppMessage::Cli(command)).await);
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
    AppIpc::send(AppMessage::Cli(vec![
        "popup".to_owned(),
        "internal-set-shortcut".to_owned(),
        serde_json::to_string(&hotkey)?,
    ]))
    .await?;
    Ok(())
}

/// Helper macro to create a command vector with optional conditional flags
///
/// Usage:
/// - Simple command: `cmd!["wm", "focus", "up"]`
/// - With variables: `cmd!["vd", "switch", index]`
/// - With conditional flags: `cmd!["task", "run"; verbose => "--verbose", debug => "--debug"]`
/// - Mixed: `cmd!["app", value; flag1 => "--flag1", cond2 => format!("--opt={}", val)]`
///
/// Examples:
/// ```
/// cmd!["settings"]
/// cmd!["vd", "switch-workspace", 3]
/// cmd!["task-switcher", "select-next"; select_on_key_up => "--auto-confirm"]
/// cmd!["wm", "run"; verbose => "--verbose", debug => "--debug", force => "--force"]
/// ```
macro_rules! cmd {
    // Simple case: just arguments, no conditional flags
    ($($arg:expr),+ $(,)?) => {
        vec![$($arg.to_string()),+]
    };
    // With conditional flags
    ($($base:expr),+ ; $($cond:expr => $flag:expr),+ $(,)?) => {
        {
            let mut v = vec![$($base.to_string()),+];
            $(
                if $cond {
                    v.push($flag.to_string());
                }
            )+
            v
        }
    };
}

fn hotkey_action_to_cli_command(action: SluHotkeyAction) -> Option<Vec<String>> {
    use SluHotkeyAction::*;

    let command = match action {
        // task switcher
        TaskNext { select_on_key_up } => {
            cmd!["task-switcher", "select-next-task"; select_on_key_up => "--auto-confirm"]
        }
        TaskPrev { select_on_key_up } => {
            cmd!["task-switcher", "select-previous-task"; select_on_key_up => "--auto-confirm"]
        }
        // Virtual Desktop
        SwitchToNextWorkspace => cmd!["vd", "switch-next"],
        SwitchToPreviousWorkspace => cmd!["vd", "switch-prev"],
        SwitchWorkspace { index } => cmd!["vd", "switch-workspace", index],
        MoveToWorkspace { index } => cmd!["vd", "move-to-workspace", index],
        SendToWorkspace { index } => cmd!["vd", "send-to-workspace", index],
        CreateNewWorkspace => cmd!["vd", "create-new-workspace"],
        DestroyCurrentWorkspace => cmd!["vd", "destroy-current-workspace"],
        ToggleWorkspacesView => cmd!["vd", "toggle-workspaces-view"],
        // wallpaper manager
        CycleWallpaperNext => cmd!["wallpaper", "next"],
        CycleWallpaperPrev => cmd!["wallpaper", "prev"],
        // Weg
        StartWegApp { index } => cmd!["weg", "foreground-or-run-app", index],
        // App Launcher / Start Menu
        ToggleLauncher => cmd!["launcher", "toggle"],
        // Window Manager
        IncreaseWidth => cmd!["wm", "width", "increase"],
        DecreaseWidth => cmd!["wm", "width", "decrease"],
        IncreaseHeight => cmd!["wm", "height", "increase"],
        DecreaseHeight => cmd!["wm", "height", "decrease"],
        RestoreSizes => cmd!["wm", "reset-workspace-size"],
        // Window Manger focused window sizing
        FocusTop => cmd!["wm", "focus", "up"],
        FocusBottom => cmd!["wm", "focus", "down"],
        FocusLeft => cmd!["wm", "focus", "left"],
        FocusRight => cmd!["wm", "focus", "right"],
        // Window Manager focused window positioning
        MoveWindowUp => cmd!["wm", "move", "up"],
        MoveWindowDown => cmd!["wm", "move", "down"],
        MoveWindowLeft => cmd!["wm", "move", "left"],
        MoveWindowRight => cmd!["wm", "move", "right"],
        // Tiling window manager reservation
        ReserveTop => cmd!["wm", "reserve", "top"],
        ReserveBottom => cmd!["wm", "reserve", "bottom"],
        ReserveLeft => cmd!["wm", "reserve", "left"],
        ReserveRight => cmd!["wm", "reserve", "right"],
        ReserveFloat => cmd!["wm", "reserve", "float"],
        ReserveStack => cmd!["wm", "reserve", "stack"],
        // Tiling window manager state
        PauseTiling => cmd!["wm", "toggle"],
        ToggleMonocle => cmd!["wm", "toggle-monocle"],
        ToggleFloat => cmd!["wm", "toggle-float"],
        CycleStackNext => cmd!["wm", "cycle-stack", "next"],
        CycleStackPrev => cmd!["wm", "cycle-stack", "prev"],
        // others
        MiscOpenSettings => cmd!["settings"],
        MiscToggleLockTracing => cmd!["debug", "toggle-trace-lock"],
        MiscToggleWinEventTracing => cmd!["debug", "toggle-win-events"],
        // no command needed
        _ => return None,
    };

    Some(command)
}
