use std::{future::Future, sync::Arc};

use interprocess::os::windows::named_pipe::{
    DuplexPipeStream, PipeListenerOptions, pipe_mode::Bytes,
    tokio::DuplexPipeStream as AsyncDuplexPipeStream,
};
use tokio::sync::mpsc;
use windows::Win32::System::RemoteDesktop::{ProcessIdToSessionId, WTSGetActiveConsoleSessionId};

use crate::{
    common::{
        IPC, read_from_ipc_stream, send_to_ipc_stream, send_to_ipc_stream_blocking,
        write_to_ipc_stream,
    },
    error::Result,
    messages::{AppMessage, IpcResponse},
    security::create_security_descriptor,
};

pub struct AppIpc {
    _priv: (),
}

impl IPC for AppIpc {
    fn path() -> String {
        let session_id = current_session_id().unwrap_or(0);
        Self::path_with_session(session_id)
    }
}

impl AppIpc {
    /// Constructs the pipe path for a specific session ID
    pub fn path_with_session(session_id: u32) -> String {
        format!(r"\\.\pipe\seelen-ui-{}", session_id)
    }

    /// Starts the IPC listener on its own thread and runtime, fully independent from the caller's
    /// runtime, so connection probes (empty messages) are always answered even if the main
    /// runtime is saturated by blocking work. Real messages are handed to the caller's runtime,
    /// which must be the current one when this function is called.
    pub fn start<R, F>(cb: F) -> Result<()>
    where
        R: Future<Output = IpcResponse> + Send + Sync + 'static,
        F: Fn(AppMessage) -> R + Send + Sync + 'static,
    {
        let main_runtime = tokio::runtime::Handle::try_current()
            .map_err(|err| std::io::Error::other(err.to_string()))?;
        let path = Self::path();
        let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<()>>();

        let callback = Arc::new(cb);
        // Notifications are acknowledged before being handled (see `process_connection`), this
        // queue keeps them in order and handles them one at a time on the main runtime.
        let (notifications_tx, mut notifications_rx) = mpsc::unbounded_channel::<AppMessage>();
        {
            let callback = callback.clone();
            main_runtime.spawn(async move {
                while let Some(message) = notifications_rx.recv().await {
                    if let IpcResponse::Err(err) = callback(message).await {
                        log::error!("Failed to process IPC notification: {err}");
                    }
                }
            });
        }

        std::thread::Builder::new()
            .name("AppIpc".into())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(err) => {
                        let _ = ready_tx.send(Err(err.into()));
                        return;
                    }
                };

                runtime.block_on(async move {
                    // the listener has to be created inside the runtime that will drive it
                    let listener = create_security_descriptor().and_then(|sd| {
                        Ok(PipeListenerOptions::new()
                            .path(path)
                            .security_descriptor(Some(sd))
                            .create_tokio_duplex::<Bytes>()?)
                    });
                    let listener = match listener {
                        Ok(listener) => listener,
                        Err(err) => {
                            let _ = ready_tx.send(Err(err));
                            return;
                        }
                    };
                    let _ = ready_tx.send(Ok(()));

                    while let Ok(stream) = listener.accept().await {
                        let callback = callback.clone();
                        let main_runtime = main_runtime.clone();
                        let notifications_tx = notifications_tx.clone();
                        tokio::spawn(async move {
                            if let Err(err) = Self::process_connection(
                                &stream,
                                callback,
                                &main_runtime,
                                &notifications_tx,
                            )
                            .await
                                && let Err(send_err) = Self::response_to_client(
                                    &stream,
                                    IpcResponse::Err(err.to_string()),
                                )
                                .await
                            {
                                log::error!(
                                    "Failed to send error response: {send_err} || Original error: {err}"
                                );
                            }
                        });
                    }
                    log::error!("AppIpc listener stopped accepting connections");
                });
            })?;

        ready_rx
            .recv()
            .map_err(|_| std::io::Error::other("AppIpc thread exited before being ready"))?
    }

    async fn process_connection<R, F>(
        stream: &AsyncDuplexPipeStream<Bytes>,
        cb: Arc<F>,
        main_runtime: &tokio::runtime::Handle,
        notifications: &mpsc::UnboundedSender<AppMessage>,
    ) -> Result<()>
    where
        R: Future<Output = IpcResponse> + Send + Sync + 'static,
        F: Fn(AppMessage) -> R + Send + Sync + 'static,
    {
        let data = read_from_ipc_stream(stream).await?;
        if data.is_empty() {
            return Self::response_to_client(stream, IpcResponse::Success).await;
        }

        let message = AppMessage::from_bytes(&data)?;
        log::trace!("IPC command received: {message:?}");

        // Notifications come from the hook DLL running on the explorer.exe tray thread, which is
        // blocked until we answer: reply first so a busy main runtime can't freeze the taskbar.
        if message.is_notification() {
            let _ = notifications.send(message);
            return Self::response_to_client(stream, IpcResponse::Success).await;
        }
        // the handler runs on the main runtime, this one must never block on it
        let response = main_runtime
            .spawn(async move { cb(message).await })
            .await
            .unwrap_or_else(|err| IpcResponse::Err(format!("IPC handler failed: {err}")));
        Self::response_to_client(stream, response).await?;
        Ok(())
    }

    async fn response_to_client(
        stream: &AsyncDuplexPipeStream<Bytes>,
        res: IpcResponse,
    ) -> Result<()> {
        write_to_ipc_stream(stream, &res.to_bytes()?).await
    }

    /// Sends a message to the current session asynchronously
    pub async fn send(message: AppMessage) -> Result<()> {
        let stream = AsyncDuplexPipeStream::connect_by_path(Self::path()).await?;
        send_to_ipc_stream(&stream, &message.to_bytes()?)
            .await?
            .ok()
    }

    /// Sends a message to the current session synchronously
    pub fn send_sync(message: &AppMessage) -> Result<()> {
        let stream = DuplexPipeStream::connect_by_path(Self::path())?;
        let data = message.to_bytes()?;
        send_to_ipc_stream_blocking(&stream, &data)?;
        Ok(())
    }
}

/// Gets the current session ID of the process
pub fn current_session_id() -> Result<u32> {
    let process_id = std::process::id();
    let mut session_id = 0;
    unsafe { ProcessIdToSessionId(process_id, &mut session_id)? };
    Ok(session_id)
}

/// Gets the current interactive session, if any
pub fn current_interactive_session_id() -> Option<u32> {
    let session_id = unsafe { WTSGetActiveConsoleSessionId() };
    if session_id == u32::MAX {
        None
    } else {
        Some(session_id)
    }
}
