use std::sync::Arc;

use interprocess::os::windows::named_pipe::{
    DuplexPipeStream, PipeListenerOptions,
    pipe_mode::Bytes,
    tokio::{DuplexPipeStream as AsyncDuplexPipeStream, PipeListenerOptionsExt},
};
use windows::Win32::System::RemoteDesktop::{ProcessIdToSessionId, WTSGetActiveConsoleSessionId};

use crate::{
    common::{
        IPC, create_security_descriptor, read_from_ipc_stream, send_to_ipc_stream,
        send_to_ipc_stream_blocking, write_to_ipc_stream,
    },
    error::Result,
    messages::{AppMessage, IpcResponse},
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

    pub fn start<F>(cb: F) -> Result<()>
    where
        F: Fn(AppMessage) -> IpcResponse + Send + Sync + 'static,
    {
        let sd = create_security_descriptor()?;

        let listener = PipeListenerOptions::new()
            .path(Self::path())
            .security_descriptor(Some(sd))
            .create_tokio_duplex::<Bytes>()?;

        tokio::spawn(async move {
            let callback = Arc::new(cb);
            while let Ok(stream) = listener.accept().await {
                let callback = callback.clone();
                tokio::spawn(async move {
                    if let Err(err) = Self::process_connection(&stream, callback).await
                        && let Err(send_err) =
                            Self::response_to_client(&stream, IpcResponse::Err(err.to_string()))
                                .await
                    {
                        log::error!(
                            "Failed to send error response: {send_err} || Original error: {err}"
                        );
                    }
                });
            }
        });
        Ok(())
    }

    async fn process_connection<F>(stream: &AsyncDuplexPipeStream<Bytes>, cb: Arc<F>) -> Result<()>
    where
        F: Fn(AppMessage) -> IpcResponse,
    {
        let data = read_from_ipc_stream(stream).await?;
        if data.is_empty() {
            return Self::response_to_client(stream, IpcResponse::Success).await;
        }

        let message = AppMessage::from_bytes(&data)?;
        log::trace!("IPC command received: {message:?}");
        Self::response_to_client(stream, cb(message)).await?;
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
