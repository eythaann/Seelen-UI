use seelen_core::{
    state::settings::media_shortcuts::parse_media_shortcut, system_state::MediaTransportAction,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT,
    KEYEVENTF_KEYUP, SendInput, VIRTUAL_KEY,
};

use crate::{error::Result, state::application::FULL_STATE, windows_api::window::Window};

pub(super) fn send_app_shortcut(
    hwnd: isize,
    expected_exe: &str,
    action: MediaTransportAction,
) -> Result<()> {
    let window = Window::from(hwnd);
    if !window.is_window() {
        return Err("The player window has closed".into());
    }
    let exe = window.process().program_exe_name()?;
    if !exe.eq_ignore_ascii_case(expected_exe) {
        return Err("The player window has changed".into());
    }
    // Snapshot configuration; do not hold state across input or a permission dialog.
    let profile = {
        let state = FULL_STATE.load();
        let profiles = &state.settings.by_widget.weg.media_shortcuts;
        profiles
            .get(&exe.to_ascii_lowercase())
            .or_else(|| {
                profiles
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case(&exe))
                    .map(|(_, profile)| profile)
            })
            .cloned()
    }
    .filter(|profile| profile.enabled)
    .ok_or("Global shortcut control is not enabled for this player")?;
    let chord = match action {
        MediaTransportAction::Previous => &profile.previous,
        MediaTransportAction::TogglePlayPause => &profile.toggle_play_pause,
        MediaTransportAction::Next => &profile.next,
    };
    let keys = parse_media_shortcut(chord)?;
    // Do not release physical keys held by the user, or alter the intended chord.
    if [0x10, 0x11, 0x12, 0x5B, 0x5C]
        .into_iter()
        .chain(keys.iter().copied())
        .any(|key| unsafe { GetAsyncKeyState(key as i32) } < 0)
    {
        return Err("Release held keys before using the player shortcut".into());
    }
    let input = |key, flags| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(key),
                dwFlags: flags,
                ..Default::default()
            },
        },
    };
    let inputs: Vec<_> = keys
        .iter()
        .map(|&key| input(key, KEYBD_EVENT_FLAGS(0)))
        .chain(keys.iter().rev().map(|&key| input(key, KEYEVENTF_KEYUP)))
        .collect();
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent != inputs.len() as u32 {
        let releases: Vec<_> = keys
            .iter()
            .rev()
            .map(|&key| input(key, KEYEVENTF_KEYUP))
            .collect();
        unsafe { SendInput(&releases, std::mem::size_of::<INPUT>() as i32) };
        return Err("Windows could not send the configured shortcut".into());
    }
    // This acknowledges input delivery, not playback state or receipt by the player.
    Ok(())
}
