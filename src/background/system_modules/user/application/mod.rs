#[cfg(windows)]
mod windows;

use std::sync::LazyLock;

use seelen_core::system_state::User;

use crate::{event_manager, utils::lock_free::TracedMutex};

use super::domain::{UserEvent, UserSystem};

#[cfg(windows)]
type Platform = windows::WindowsUser;

pub struct UserManager {
    platform: Platform,
    /// Last known user, used to answer queries and to only notify real changes.
    user: TracedMutex<User>,
}

event_manager!(UserManager, UserEvent);

impl UserManager {
    /// Building the user data is just a handful of registry reads, so it's fine to build
    /// it lazily on first access instead of needing a background thread + placeholder.
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<UserManager> = LazyLock::new(|| {
            let m = UserManager::create();
            m.init();
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
            user: TracedMutex::new(User::default()),
        }
    }

    /// The OS doesn't notify user changes, they are picked up by [`Self::refresh`].
    fn init(&self) {
        *self.user.lock() = self.platform.logged_user();
    }

    pub fn user(&self) -> User {
        self.user.lock().clone()
    }

    /// Reads the user again, notifying it when changed.
    pub fn refresh(&self) -> User {
        let user = self.platform.logged_user();
        {
            let mut current = self.user.lock();
            if *current == user {
                return user;
            }
            *current = user.clone();
        }
        Self::send(UserEvent::Changed { user: user.clone() });
        user
    }
}
