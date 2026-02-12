use std::{future::Future, sync::Arc};

use interprocess::os::windows::named_pipe::{
    PipeListenerOptions, pipe_mode::Bytes, tokio::DuplexPipeStream as AsyncDuplexPipeStream,
};

use crate::{
    app::current_session_id,
    common::{
        IPC, create_security_descriptor, read_from_ipc_stream, send_to_ipc_stream, send_with_retry,
        write_to_ipc_stream,
    },
    error::Result,
    messages::{IpcResponse, SvcAction, SvcMessage},
};

pub struct ServiceIpc {
    _priv: (),
}

impl IPC for ServiceIpc {
    fn path() -> String {
        let session_id = current_session_id().unwrap_or(0);
        Self::path_with_session(session_id)
    }
}

impl ServiceIpc {
    /// Constructs the pipe path for a specific session ID
    pub fn path_with_session(session_id: u32) -> String {
        format!(r"\\.\pipe\seelen-ui-service-{}", session_id)
    }
}

impl ServiceIpc {
    pub fn start<R, F>(cb: F) -> Result<()>
    where
        R: Future<Output = IpcResponse> + Send + Sync,
        F: Fn(SvcAction) -> R + Send + Sync + 'static,
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

    async fn process_connection<F, R>(
        stream: &AsyncDuplexPipeStream<Bytes>,
        cb: Arc<F>,
    ) -> Result<()>
    where
        R: Future<Output = IpcResponse> + Send + Sync,
        F: Fn(SvcAction) -> R + Send + Sync + 'static,
    {
        let data = read_from_ipc_stream(stream).await?;
        if data.is_empty() {
            return Self::response_to_client(stream, IpcResponse::Success).await;
        }

        let message = SvcMessage::from_bytes(&data)?;
        if !message.is_signature_valid() {
            Self::response_to_client(
                stream,
                IpcResponse::Err("Unauthorized connection".to_owned()),
            )
            .await?;
            return Ok(());
        }

        log::trace!("IPC command received: {:?}", message.action);
        Self::response_to_client(stream, cb(message.action).await).await?;
        Ok(())
    }

    async fn response_to_client(
        stream: &AsyncDuplexPipeStream<Bytes>,
        res: IpcResponse,
    ) -> Result<()> {
        write_to_ipc_stream(stream, &res.to_bytes()?).await
    }

    pub async fn send(message: SvcAction) -> Result<()> {
        let data = SvcMessage {
            token: SvcMessage::signature().to_string(),
            action: message,
        }
        .to_bytes()?;

        send_with_retry(|| Self::try_send(&data)).await
    }

    async fn try_send(data: &[u8]) -> Result<IpcResponse> {
        let stream = AsyncDuplexPipeStream::connect_by_path(Self::path()).await?;
        send_to_ipc_stream(&stream, data).await
    }
}
