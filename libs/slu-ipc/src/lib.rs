pub mod error;
pub mod messages;

use std::{
    io::{BufRead, Write},
    sync::Arc,
};

use interprocess::os::windows::{
    named_pipe::{
        DuplexPipeStream, PipeListenerOptions,
        pipe_mode::Bytes,
        tokio::{DuplexPipeStream as AsyncDuplexPipeStream, PipeListenerOptionsExt},
    },
    security_descriptor::{AsSecurityDescriptorMutExt, SecurityDescriptor},
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

use crate::{
    error::Result,
    messages::{AppMessage, IpcResponse, LauncherMessage, SvcAction, SvcMessage},
};

/// https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-control
static SE_DACL_PROTECTED: u16 = 4096u16;

// const END_OF_TRANSMISSION: u8 = 0x04;
const END_OF_TRANSMISSION_BLOCK: u8 = 0x17;

pub trait IPC {
    const PATH: &'static str;

    #[allow(async_fn_in_trait)]
    async fn server_process_id() -> Result<u32> {
        let stream = AsyncDuplexPipeStream::connect_by_path(Self::PATH).await?;
        let pid = stream.server_process_id()?;
        write_to_ipc_stream(&stream, &[]).await?;
        Ok(pid)
    }

    /// returns the server process id
    fn test_connection() -> Result<()> {
        let stream = DuplexPipeStream::connect_by_path(Self::PATH)?;
        let response = send_to_ipc_stream(&stream, &[])?;
        response.ok()
    }

    fn can_stablish_connection() -> bool {
        Self::test_connection().is_ok()
    }
}

pub struct ServiceIpc {
    _priv: (),
}

impl IPC for ServiceIpc {
    const PATH: &'static str = r"\\.\pipe\seelen-ui-service";
}

impl ServiceIpc {
    pub fn start<R, F>(cb: F) -> Result<()>
    where
        R: Future<Output = IpcResponse> + Send + Sync,
        F: Fn(SvcAction) -> R + Send + Sync + 'static,
    {
        let mut sd = SecurityDescriptor::new()?;
        unsafe { sd.set_dacl(std::ptr::null_mut(), false)? };
        sd.set_control(SE_DACL_PROTECTED, SE_DACL_PROTECTED)?;

        let listener = PipeListenerOptions::new()
            .path(Self::PATH)
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
        let stream = AsyncDuplexPipeStream::connect_by_path(Self::PATH).await?;
        let data = SvcMessage {
            token: SvcMessage::signature().to_string(),
            action: message,
        }
        .to_bytes()?;
        async_send_to_ipc_stream(&stream, &data).await?.ok()
    }
}

pub struct AppIpc {
    _priv: (),
}

impl IPC for AppIpc {
    const PATH: &'static str = r"\\.\pipe\seelen-ui";
}

impl AppIpc {
    pub fn start<F>(cb: F) -> Result<()>
    where
        F: Fn(Vec<String>) -> IpcResponse + Send + Sync + 'static,
    {
        let mut sd = SecurityDescriptor::new()?;
        unsafe { sd.set_dacl(std::ptr::null_mut(), false)? };
        sd.set_control(SE_DACL_PROTECTED, SE_DACL_PROTECTED)?;

        let listener = PipeListenerOptions::new()
            .path(Self::PATH)
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
        F: Fn(Vec<String>) -> IpcResponse,
    {
        let data = read_from_ipc_stream(stream).await?;
        if data.is_empty() {
            return Self::response_to_client(stream, IpcResponse::Success).await;
        }

        let message = AppMessage::from_bytes(&data)?;
        log::trace!("IPC command received: {message:?}");
        Self::response_to_client(stream, cb(message.0)).await?;
        Ok(())
    }

    async fn response_to_client(
        stream: &AsyncDuplexPipeStream<Bytes>,
        res: IpcResponse,
    ) -> Result<()> {
        write_to_ipc_stream(stream, &res.to_bytes()?).await
    }

    pub async fn send(message: AppMessage) -> Result<()> {
        let stream = AsyncDuplexPipeStream::connect_by_path(Self::PATH).await?;
        async_send_to_ipc_stream(&stream, &message.to_bytes()?)
            .await?
            .ok()
    }
}

async fn read_from_ipc_stream(stream: &AsyncDuplexPipeStream<Bytes>) -> Result<Vec<u8>> {
    let mut reader = BufReader::new(stream);
    let mut buf = Vec::new();
    reader
        .read_until(END_OF_TRANSMISSION_BLOCK, &mut buf)
        .await?;
    buf.pop();
    Ok(buf)
}

async fn write_to_ipc_stream(stream: &AsyncDuplexPipeStream<Bytes>, buf: &[u8]) -> Result<()> {
    let mut writter = BufWriter::new(stream);
    writter.write_all(buf).await?;
    writter.write_all(&[END_OF_TRANSMISSION_BLOCK]).await?;
    writter.flush().await?;
    Ok(())
}

async fn async_send_to_ipc_stream(
    stream: &AsyncDuplexPipeStream<Bytes>,
    buf: &[u8],
) -> Result<IpcResponse> {
    write_to_ipc_stream(stream, buf).await?;
    let buf = read_from_ipc_stream(stream).await?;
    IpcResponse::from_bytes(&buf)
}

/// blocking version to test connections without needed of tokio runtime
fn send_to_ipc_stream(stream: &DuplexPipeStream<Bytes>, buf: &[u8]) -> Result<IpcResponse> {
    let mut writter = std::io::BufWriter::new(stream);
    writter.write_all(buf)?;
    writter.write_all(&[END_OF_TRANSMISSION_BLOCK])?;
    writter.flush()?;

    let mut reader = std::io::BufReader::new(stream);
    let mut buf = Vec::new();
    reader.read_until(END_OF_TRANSMISSION_BLOCK, &mut buf)?;
    buf.pop();

    IpcResponse::from_bytes(&buf)
}

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
        let mut sd = SecurityDescriptor::new()?;
        unsafe { sd.set_dacl(std::ptr::null_mut(), false)? };
        sd.set_control(SE_DACL_PROTECTED, SE_DACL_PROTECTED)?;

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
        async_send_to_ipc_stream(&stream, &message.to_bytes()?)
            .await?
            .ok()
    }
}
