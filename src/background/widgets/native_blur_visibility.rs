//! Per-document visibility ordering, independent of geometry request ordering.
use std::time::Instant;

#[derive(Default)]
pub(super) struct Visibility {
    pub epoch: u32,
    pub deadline: Option<Instant>,
    switch_suppressed: bool,
}

impl Visibility {
    pub fn open(&mut self, epoch: u32) -> bool {
        // Close wins within a generation; only an actual new show may reopen it.
        if epoch < self.epoch || (epoch == self.epoch && self.deadline.is_some()) {
            return false;
        }
        if self.switch_suppressed {
            return false;
        }
        self.epoch = epoch;
        self.deadline = None;
        true
    }

    pub fn close(&mut self, epoch: u32, deadline: Instant) -> Option<Instant> {
        if epoch < self.epoch {
            return None;
        }
        self.deadline = Some(if epoch == self.epoch {
            self.deadline
                .map_or(deadline, |previous| previous.min(deadline))
        } else {
            deadline
        });
        self.epoch = epoch;
        self.deadline
    }

    pub fn accept_frame(&mut self, epoch: u32, now: Instant) -> bool {
        // A geometry frame alone is not proof that a switched-away popup reopened.
        if self.switch_suppressed {
            return false;
        }
        if epoch < self.epoch {
            return false;
        }
        if epoch > self.epoch {
            self.open(epoch);
        }
        !self.deadline.is_some_and(|deadline| now >= deadline)
    }

    pub fn expired(&self, epoch: u32, now: Instant) -> bool {
        self.epoch == epoch && self.deadline.is_some_and(|deadline| now >= deadline)
    }

    pub fn suppress_for_popup_switch(&mut self, now: Instant) {
        self.switch_suppressed = true;
        self.close(self.epoch, now);
    }

    pub fn allow_popup_trigger(&mut self) {
        // Only a new backend trigger may lift a switch barrier. A delayed open
        // from the old show must not restore its visuals during the next load.
        self.switch_suppressed = false;
    }

    pub fn may_restore_focus_visual(&self) -> bool {
        !self.switch_suppressed && self.deadline.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn stale_geometry_and_old_close_cannot_resurrect_or_erase_a_new_show() {
        let now = Instant::now();
        let mut state = Visibility::default();
        state.close(2, now + Duration::from_millis(80));
        assert!(!state.accept_frame(1, now));
        assert!(state.accept_frame(2, now));
        assert!(!state.accept_frame(2, now + Duration::from_millis(90)));
        assert!(state.open(3));
        assert!(state.close(2, now).is_none());
        assert!(!state.expired(2, now + Duration::from_secs(1)));
        assert!(state.accept_frame(3, now));
    }

    #[test]
    fn delayed_begin_close_cannot_extend_an_already_completed_close() {
        let now = Instant::now();
        let mut state = Visibility::default();
        state.close(1, now);
        state.close(1, now + Duration::from_millis(200));
        assert!(state.expired(1, now));
        assert!(!state.open(1));
        assert!(state.open(2));
    }

    #[test]
    fn popup_switch_blocks_late_geometry_until_explicit_new_show() {
        let now = Instant::now();
        let mut state = Visibility::default();
        assert!(state.open(4));
        state.suppress_for_popup_switch(now);
        assert!(!state.accept_frame(4, now));
        assert!(!state.accept_frame(5, now));
        assert!(!state.open(4));
        assert!(!state.open(5));
        state.close(5, now); // The old document eventually sends its close.
        assert!(!state.accept_frame(6, now));
        state.allow_popup_trigger();
        assert!(state.open(6));
        assert!(state.accept_frame(6, now));
        assert!(state.close(5, now).is_none());
        assert!(!state.expired(5, now));
    }

    #[test]
    fn repeated_popup_switch_and_delayed_close_cannot_erase_reopened_popup() {
        let now = Instant::now();
        let mut state = Visibility::default();
        for epoch in 1..=500 {
            state.allow_popup_trigger();
            assert!(state.open(epoch));
            assert!(state.accept_frame(epoch, now));
            assert!(!state.expired(epoch - 1, now));
            assert!(state.close(epoch - 1, now).is_none());
            state.suppress_for_popup_switch(now);
            state.suppress_for_popup_switch(now);
            assert!(!state.accept_frame(epoch, now));
            assert!(!state.accept_frame(epoch + 1, now));
        }
    }

    #[test]
    fn blur_focus_return_must_not_override_a_real_close_or_popup_switch() {
        let now = Instant::now();
        let mut state = Visibility::default();
        state.open(4);
        assert!(state.may_restore_focus_visual());
        state.close(5, now);
        assert!(!state.may_restore_focus_visual());
        state.open(6);
        state.suppress_for_popup_switch(now);
        assert!(!state.may_restore_focus_visual());
        state.allow_popup_trigger();
        assert!(!state.may_restore_focus_visual());
        assert!(state.open(7));
        assert!(state.may_restore_focus_visual());
    }
}
