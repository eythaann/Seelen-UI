//! Reversible native-only focus gate. Does not advance document visibility epochs.
#[derive(Default)]
pub(super) struct PopupFocus {
    owned: bool,
    suppressed: bool,
}

impl PopupFocus {
    pub fn new(hwnd: isize, focused: isize, owner: isize) -> Self {
        Self {
            owned: hwnd == focused || hwnd == owner,
            suppressed: false,
        }
    }

    /// Some(false) hides; Some(true) restores after a canceled focus-loss close.
    pub fn observe(&mut self, hwnd: isize, focused: isize, owner: isize) -> Option<bool> {
        if focused == 0 {
            return None;
        }
        let owned = hwnd == focused || hwnd == owner;
        let previous = self.owned;
        self.owned = owned;
        if owned && self.suppressed {
            self.suppressed = false;
            Some(true)
        } else if previous && !owned {
            self.suppressed = true;
            Some(false)
        } else {
            None
        }
    }

    pub fn suppressed(&self) -> bool {
        self.suppressed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blur_focus_loss_and_return_are_reversible_without_epoch_changes() {
        let mut state = PopupFocus::new(11, 11, 0);
        assert_eq!(state.observe(11, 22, 0), Some(false));
        assert!(state.suppressed());
        assert_eq!(state.observe(11, 22, 0), None);
        assert_eq!(state.observe(11, 11, 0), Some(true));
        assert!(!state.suppressed());
    }

    #[test]
    fn blur_focus_owned_child_keeps_parent_visual_and_repeated_focus_is_noop() {
        let mut state = PopupFocus::new(11, 11, 0);
        assert_eq!(state.observe(11, 33, 11), None);
        assert_eq!(state.observe(11, 11, 0), None);
        assert!(!state.suppressed());
        assert_eq!(state.observe(11, 44, 0), Some(false));
    }

    #[test]
    fn blur_focus_unseen_or_transient_foreground_does_not_hide_unrelated_popup() {
        let mut state = PopupFocus::new(11, 22, 0);
        assert_eq!(state.observe(11, 44, 0), None);
        assert_eq!(state.observe(11, 0, 0), None);
        assert!(!state.suppressed());
        assert_eq!(state.observe(11, 11, 0), None);
        assert_eq!(state.observe(11, 22, 0), Some(false));
    }
}
