use std::sync::Arc;

use interprocess::os::windows::named_pipe::{
    PipeListenerOptions,
    pipe_mode::Bytes,
    tokio::{DuplexPipeStream as AsyncDuplexPipeStream, PipeListenerOptionsExt},
};

use crate::{
    common::{
        IPC, create_security_descriptor, read_from_ipc_stream, send_to_ipc_stream,
        write_to_ipc_stream,
    },
    error::Result,
    messages::{IpcResponse, LauncherMessage},
};

pub struct LauncherIpc {
    _priv: (),
}

impl IPC for LauncherIpc {
    const PATH: &'static str = r"\\.\pipe\seelen-ui-launcher";
}

impl LauncherIpc {
    pub fn start<F>(cb: F) -> Result<()>
    where
        F: Fn(LauncherMessage) -> IpcResponse + Send + Sync + 'static,
    {
        let sd = create_security_descriptor()?;

        let listener = PipeListenerOptions::new()
            .path(Self::PATH)
            .security_descriptor(Some(sd))
            .create_tokio_duplex::<Bytes>()?;

        tokio::spawn(async move {
            let cb = Arc::new(cb);
            while let Ok(stream) = listener.accept().await {
                let cb = cb.clone();
                tokio::spawn(async move {
                    if let Err(err) = Self::process_connection(&stream, cb).await
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

    async fn response_to_client(
        stream: &AsyncDuplexPipeStream<Bytes>,
        res: IpcResponse,
    ) -> Result<()> {
        write_to_ipc_stream(stream, &res.to_bytes()?).await
    }

    async fn process_connection<F>(stream: &AsyncDuplexPipeStream<Bytes>, cb: Arc<F>) -> Result<()>
    where
        F: Fn(LauncherMessage) -> IpcResponse + Send + Sync,
    {
        let data = read_from_ipc_stream(stream).await?;
        if data.is_empty() {
            return Self::response_to_client(stream, IpcResponse::Success).await;
        }
        let message = LauncherMessage::from_bytes(&data)?;
        log::trace!("IPC command received: {message:?}");
        Self::response_to_client(stream, cb(message)).await?;
        Ok(())
    }

    pub async fn send(message: LauncherMessage) -> Result<()> {
        let stream = AsyncDuplexPipeStream::connect_by_path(Self::PATH).await?;
        send_to_ipc_stream(&stream, &message.to_bytes()?)
            .await?
            .ok()
    }
}
