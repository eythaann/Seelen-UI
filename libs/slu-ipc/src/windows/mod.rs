mod security;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::RecvTimeoutError,
    },
    time::Duration,
};

use interprocess::{
    local_socket::{ListenerOptions, Stream},
    os::windows::local_socket::ListenerOptionsExt,
};
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::{
        IO::CancelSynchronousIo,
        RemoteDesktop::{ProcessIdToSessionId, WTSGetActiveConsoleSessionId},
        Threading::{GetCurrentThreadId, OpenThread, THREAD_TERMINATE},
    },
};

use crate::{
    domain::IpcSystem,
    error::{Error, Result},
};

/// Local sockets are named pipes (byte mode, duplex) on Windows.
pub struct WindowsIpc;

impl IpcSystem for WindowsIpc {
    fn current_session_id() -> Result<u32> {
        let mut session_id = 0;
        unsafe { ProcessIdToSessionId(std::process::id(), &mut session_id)? };
        Ok(session_id)
    }

    fn interactive_session_id() -> Option<u32> {
        let session_id = unsafe { WTSGetActiveConsoleSessionId() };
        (session_id != u32::MAX).then_some(session_id)
    }

    fn endpoint(name: &str, session_id: u32) -> String {
        format!(r"\\.\pipe\{name}-{session_id}")
    }

    fn secure_listener(options: ListenerOptions<'_>) -> Result<ListenerOptions<'_>> {
        Ok(options.security_descriptor(security::create_security_descriptor()?))
    }

    /// Named pipe local sockets have no read/write timeouts, so the exchange is cancelled from a
    /// watchdog thread with `CancelSynchronousIo`.
    fn with_io_timeout<T>(
        _stream: &Stream,
        timeout: Duration,
        io: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        let thread = ThreadHandle::current()?;
        let timed_out = AtomicBool::new(false);

        let result = std::thread::scope(|scope| {
            let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
            let (thread, timed_out) = (&thread, &timed_out);
            scope.spawn(move || {
                if let Err(RecvTimeoutError::Timeout) = done_rx.recv_timeout(timeout) {
                    timed_out.store(true, Ordering::Release);
                    // retry until the exchange finishes, the I/O could start right after a cancel
                    while let Err(RecvTimeoutError::Timeout) =
                        done_rx.recv_timeout(Duration::from_millis(10))
                    {
                        thread.cancel_io();
                    }
                }
            });
            let result = io();
            drop(done_tx);
            result
        });

        if timed_out.load(Ordering::Acquire) && result.is_err() {
            return Err(Error::Timeout("No response from IPC stream".to_string()));
        }
        result
    }
}

/// Owned handle of a thread, stored as address because `HANDLE` is not `Send`.
struct ThreadHandle(isize);

impl ThreadHandle {
    fn current() -> Result<Self> {
        // CancelSynchronousIo needs a real handle with THREAD_TERMINATE access, not the pseudo one
        let thread = unsafe { OpenThread(THREAD_TERMINATE, false, GetCurrentThreadId())? };
        Ok(Self(thread.0 as isize))
    }

    fn cancel_io(&self) {
        let _ = unsafe { CancelSynchronousIo(HANDLE(self.0 as _)) };
    }
}

impl Drop for ThreadHandle {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(HANDLE(self.0 as _)) };
    }
}
