use std::sync::Arc;

use seelen_core::state::shortcuts::ResolvedShortcut;
use win_hotkeys::{
    Hotkey, HotkeyManager, TriggerTiming, VKey, error::WHKError, events::KeyboardInputEvent,
};

use crate::{
    error::Result,
    log_error,
    modules::hotkeys::domain::{HotkeysSystem, RecordingEvent},
};

pub struct WindowsHotkeys;

impl HotkeysSystem for WindowsHotkeys {
    fn create() -> Self {
        Self
    }

    fn register<F>(&self, shortcuts: Vec<ResolvedShortcut>, on_trigger: F) -> Result<()>
    where
        F: Fn(&[String]) + Clone + Send + Sync + 'static,
    {
        if let Err(err) = HotkeyManager::start_keyboard_capturing() {
            match err {
                WHKError::AlreadyStarted => {}
                others => return Err(others.into()),
            }
        };

        let manager = HotkeyManager::current();
        manager.unregister_all()?;

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

            let command = s.command;
            let on_trigger = on_trigger.clone();
            let mut hotkey = Hotkey::from_keys(&vkeys).action(move || on_trigger(&command));

            if vkeys.len() == 1 {
                hotkey.trigger_timing = TriggerTiming::OnKeyUp;
                hotkey.strict_sequence = true;
            }

            log_error!(manager.register_hotkey(hotkey));
        }
        Ok(())
    }

    fn stop(&self) {
        HotkeyManager::stop_keyboard_capturing();
    }

    fn start_recording<F>(&self, on_event: F) -> Result<()>
    where
        F: Fn(RecordingEvent) + Send + Sync + 'static,
    {
        let on_event = Arc::new(on_event);
        let hkm = HotkeyManager::current();

        let on_finished = on_event.clone();
        hkm.steal_keyboard(move || {
            on_finished(RecordingEvent::Finished);
            HotkeyManager::current().remove_global_keyboard_listener();
        });

        hkm.set_global_keyboard_listener(move |event| match event {
            // escape cancels the recording, it is not part of the shortcut
            KeyboardInputEvent::KeyDown { key, state } if key != VKey::Escape => {
                let keys = state.pressing.iter().map(|vkey| vkey.to_string()).collect();
                on_event(RecordingEvent::Pressed { keys });
            }
            _ => {}
        });
        Ok(())
    }

    fn stop_recording(&self) {
        HotkeyManager::current().free_keyboard();
    }
}
