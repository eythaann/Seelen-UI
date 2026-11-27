use seelen_core::state::shortcuts::ResolvedShortcut;

use crate::error::Result;

/// OS contract of the hotkeys module.
pub trait HotkeysSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here.
    fn create() -> Self;

    /// Starts capturing the keyboard (if needed) and replaces the registered shortcuts.
    /// Shortcuts whose keys are unknown are skipped. `on_trigger` receives the command of the
    /// triggered shortcut.
    fn register<F>(&self, shortcuts: Vec<ResolvedShortcut>, on_trigger: F) -> Result<()>
    where
        F: Fn(&[String]) + Clone + Send + Sync + 'static;

    /// Stops capturing the keyboard: no shortcut is triggered anymore.
    fn stop(&self);

    /// Takes the keyboard to record a new shortcut: keys are not delivered to the other apps
    /// and no shortcut is triggered until the recording finishes, reported through `on_event`.
    fn start_recording<F>(&self, on_event: F) -> Result<()>
    where
        F: Fn(RecordingEvent) + Send + Sync + 'static;

    /// Finishes the recording, giving the keyboard back.
    fn stop_recording(&self);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingEvent {
    /// Keys being pressed right now, by name.
    Pressed { keys: Vec<String> },
    /// The recording finished (cancelled by the user or stopped by the app).
    Finished,
}

/// What a triggered shortcut does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutAction {
    /// Kills the app, the service monitoring launches it again.
    RestartApp,
    /// Kills the app and stops the service.
    Quit,
    /// A command handled by the app itself.
    ForwardToApp,
}

impl ShortcutAction {
    pub fn of(command: &[String]) -> Self {
        match command {
            [a, b] if a == "service" && b == "force-restart" => Self::RestartApp,
            [a, b] if a == "service" && b == "force-quit" => Self::Quit,
            _ => Self::ForwardToApp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|p| p.to_string()).collect()
    }

    #[test]
    fn service_commands_are_handled_by_the_service() {
        assert_eq!(
            ShortcutAction::of(&command(&["service", "force-restart"])),
            ShortcutAction::RestartApp
        );
        assert_eq!(
            ShortcutAction::of(&command(&["service", "force-quit"])),
            ShortcutAction::Quit
        );
        assert_eq!(
            ShortcutAction::of(&command(&["service", "force-quit", "now"])),
            ShortcutAction::ForwardToApp
        );
        assert_eq!(
            ShortcutAction::of(&command(&["weg", "foreground-or-run-app", "0"])),
            ShortcutAction::ForwardToApp
        );
    }
}
