use seelen_core::system_state::{ImeState, SystemLanguage};

use crate::error::Result;

/// OS contract of the input languages module (languages, keyboard layouts and IME).
pub trait LanguagesSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here.
    fn create() -> Self;

    /// Starts watching the active layout and IME state, reported through `on_event`.
    /// Called once; the watching lives for the app lifetime.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(LanguagesSysEvent) + Send + Sync + 'static;

    /// Languages of the user, with their enabled keyboard layouts / input methods.
    fn languages(&self) -> Result<Vec<SystemLanguage>>;

    /// IME state of the focused window.
    fn ime_state(&self) -> Result<ImeState>;

    /// Activates a keyboard layout (`id`) or input method (`handle`) of a language.
    fn set_keyboard_layout(&self, id: &str, handle: &str) -> Result<()>;
}

/// Raw OS notifications: they only signal what changed, the manager queries the new data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguagesSysEvent {
    /// The active layout changed (or the layouts list).
    LayoutChanged,
    ImeChanged,
}

#[derive(Debug, Clone)]
pub enum LanguageEvent {
    LayoutChanged,
    ImeChanged,
}
