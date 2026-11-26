use std::{future::Future, sync::Arc};

use interprocess::local_socket::traits::tokio::Listener as _;

use crate::{
    AsyncIpcStream, Platform,
    app::current_session_id,
    common::{
        IPC, connect, connect_async, listen, read_from_ipc_stream, send_to_ipc_stream,
        send_to_ipc_stream_blocking, send_with_retry, write_to_ipc_stream,
    },
    domain::IpcSystem,
    error::Result,
    messages::{IpcResponse, SvcAction},
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
    /// Constructs the endpoint path for a specific session ID
    pub fn path_with_session(session_id: u32) -> String {
        Platform::endpoint("seelen-ui-service", session_id)
    }
}

impl ServiceIpc {
    pub fn start<R, F>(cb: F) -> Result<()>
    where
        R: Future<Output = IpcResponse> + Send + Sync,
        F: Fn(SvcAction) -> R + Send + Sync + 'static,
    {
        let listener = listen(&Self::path())?;

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

    async fn process_connection<F, R>(stream: &AsyncIpcStream, cb: Arc<F>) -> Result<()>
    where
        R: Future<Output = IpcResponse> + Send + Sync,
        F: Fn(SvcAction) -> R + Send + Sync + 'static,
    {
        let data = read_from_ipc_stream(stream).await?;
        if data.is_empty() {
            return Self::response_to_client(stream, IpcResponse::Success).await;
        }

        let action = SvcAction::from_bytes(&data)?;
        log::trace!("IPC command received: {action:?}");
        Self::response_to_client(stream, cb(action).await).await?;
        Ok(())
    }

    async fn response_to_client(stream: &AsyncIpcStream, res: IpcResponse) -> Result<()> {
        write_to_ipc_stream(stream, &res.to_bytes()?).await
    }

    pub async fn send(message: SvcAction) -> Result<()> {
        let data = message.to_bytes()?;
        send_with_retry(|| Self::try_send(&data)).await
    }

    pub fn send_blocking(message: SvcAction) -> Result<IpcResponse> {
        let data = message.to_bytes()?;
        let stream = connect(&Self::path())?;
        send_to_ipc_stream_blocking(&stream, &data)
    }

    async fn try_send(data: &[u8]) -> Result<IpcResponse> {
        let stream = connect_async(&Self::path()).await?;
        send_to_ipc_stream(&stream, data).await
    }
}
