//! OS contract of the crate. The transport itself is `interprocess::local_socket` (named pipes on
//! Windows, Unix domain sockets on Unix) and lives in the OS-agnostic code; this trait only covers
//! what local sockets leave to each OS: where the endpoints live, who may connect to them, how a
//! blocking exchange is bounded in time and the OS user session they are scoped to.

use std::time::Duration;

use interprocess::local_socket::{ListenerOptions, Stream};

use crate::error::Result;

/// Stateless: every item is an associated function.
pub trait IpcSystem {
    /// OS user session of this process.
    fn current_session_id() -> Result<u32>;

    /// Session attached to the physical console, if any.
    fn interactive_session_id() -> Option<u32>;

    /// Filesystem path of the endpoint `name` scoped to `session_id`, as accepted by
    /// `GenericFilePath`.
    fn endpoint(name: &str, session_id: u32) -> String;

    /// Restricts the listener to the user of the current session, the OS itself and elevated
    /// administrators.
    fn secure_listener(options: ListenerOptions<'_>) -> Result<ListenerOptions<'_>>;

    /// Runs the blocking exchange `io` on `stream`, aborting it with [`crate::error::Error::Timeout`]
    /// when it takes longer than `timeout`.
    fn with_io_timeout<T>(
        stream: &Stream,
        timeout: Duration,
        io: impl FnOnce() -> Result<T>,
    ) -> Result<T>;
}
