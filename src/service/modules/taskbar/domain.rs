use crate::error::Result;

/// OS contract of the taskbar module.
pub trait TaskbarSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here.
    fn create() -> Self;

    /// Hides the native taskbars that are shown right now (one check of a hide pass).
    ///
    /// - `may_move_focus`: whether it may move the focus out of a focused taskbar, which keeps
    ///   it shown. Expensive, so it is only allowed until it was done once in the pass.
    /// - `still_hidden`: re-checked after slow steps, nothing is hidden once it returns `false`
    ///   (a restore arrived meanwhile).
    ///
    /// Returns whether the focus was moved.
    fn hide<F>(&self, may_move_focus: bool, still_hidden: F) -> Result<bool>
    where
        F: Fn() -> bool;

    /// Shows the native taskbars again with their default behavior.
    fn restore(&self) -> Result<()>;
}
