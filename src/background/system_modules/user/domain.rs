use seelen_core::system_state::User;

/// OS contract of the user module.
pub trait UserSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here.
    fn create() -> Self;

    /// Reads the logged user from the OS. Never fails: missing data is left empty.
    fn logged_user(&self) -> User;
}

#[derive(Debug, Clone)]
pub enum UserEvent {
    Changed { user: User },
}
