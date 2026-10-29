//! Per-document visibility ordering, independent of geometry request ordering.
use std::time::Instant;

#[derive(Default)]
pub(super) struct Visibility {
    pub epoch: u32,
    pub deadline: Option<Instant>,
}

impl Visibility {
    pub fn open(&mut self, epoch: u32) -> bool {
        // Close wins within a generation; only an actual new show may reopen it.
        if epoch < self.epoch || (epoch == self.epoch && self.deadline.is_some()) {
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
}
