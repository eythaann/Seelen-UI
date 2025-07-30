pub mod error;
pub mod messages;

use std::{
    io::{BufRead, Write},
    sync::Arc,
};

use interprocess::os::windows::{
    named_pipe::{
        PipeListenerOptions, PipeStream,
        pipe_mode::Bytes,
        tokio::{DuplexPipeStream, PipeListenerOptionsExt},
    },
    security_descriptor::{AsSecurityDescriptorMutExt, SecurityDescriptor},
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

use crate::{
    error::Result,
    messages::{IpcResponse, SvcAction, SvcMessage},
};

/// https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-control
static SE_DACL_PROTECTED: u16 = 4096u16;

pub trait IPC {
    const PATH: &'static str;
    const END_OF_TRANSMISSION_BLOCK: u8 = 0x17;

    fn test_connection() -> Result<()> {
        let mut stream = PipeStream::<Bytes, Bytes>::connect_by_path(Self::PATH)?;
        stream.write_all(&[Self::END_OF_TRANSMISSION_BLOCK])?;
        stream.flush()?;

        let mut reader = std::io::BufReader::new(&stream);
        let mut data = Vec::new();
        reader.read_until(Self::END_OF_TRANSMISSION_BLOCK, &mut data)?;
        data.pop();

        let response: IpcResponse = serde_json::from_slice(&data)?;
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
    pub fn start<F>(cb: F) -> Result<()>
    where
        F: Fn(SvcAction) -> IpcResponse + Send + Sync + 'static,
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

    async fn process_connection<F>(stream: &DuplexPipeStream<Bytes>, cb: Arc<F>) -> Result<()>
    where
        F: Fn(SvcAction) -> IpcResponse,
    {
        let mut reader = BufReader::new(stream);
        let mut data = Vec::new();
        reader.read_until(0x17, &mut data).await?;
        data.pop(); // Remove end of transmission block

        if data.is_empty() {
            return Self::response_to_client(stream, IpcResponse::Success).await;
        }

        let message: SvcMessage = serde_json::from_slice(&data)?;
        if !message.is_signature_valid() {
            Self::response_to_client(
                stream,
                IpcResponse::Err("Unauthorized connection".to_owned()),
            )
            .await?;
            return Ok(());
        }

        log::trace!("IPC command received: {:?}", message.action);
        Self::response_to_client(stream, cb(message.action)).await?;
        Ok(())
    }

    async fn response_to_client(stream: &DuplexPipeStream<Bytes>, res: IpcResponse) -> Result<()> {
        let message = serde_json::to_vec(&res)?;

        let mut writer = BufWriter::new(stream);
        writer.write_all(&message).await?;
        writer.write_all(&[0x17]).await?;
        writer.flush().await?;
        Ok(())
    }

    pub async fn send(message: SvcAction) -> Result<()> {
        let stream = DuplexPipeStream::connect_by_path(Self::PATH).await?;
        let data = serde_json::to_vec(&SvcMessage {
            token: SvcMessage::signature().to_string(),
            action: message,
        })?;

        {
            let mut writer = BufWriter::new(&stream);
            writer.write_all(&data).await?;
            writer.write_all(&[0x17]).await?;
            writer.flush().await?;
        }

        {
            let mut reader = BufReader::new(&stream);
            let mut data = Vec::new();
            reader.read_until(0x17, &mut data).await?; // Read until end of transmission block
            data.pop(); // Remove end of transmission block

            let response: IpcResponse = serde_json::from_slice(&data)?;
            response.ok()
        }
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

    async fn process_connection<F>(stream: &DuplexPipeStream<Bytes>, cb: Arc<F>) -> Result<()>
    where
        F: Fn(Vec<String>) -> IpcResponse,
    {
        let mut reader = BufReader::new(stream);
        let mut data = Vec::new();
        reader.read_until(0x17, &mut data).await?;
        data.pop(); // Remove end of transmission block

        if data.is_empty() {
            return Self::response_to_client(stream, IpcResponse::Success).await;
        }

        let message: Vec<String> = serde_json::from_slice(&data)?;
        log::trace!("IPC command received: {message:?}");
        Self::response_to_client(stream, cb(message)).await?;
        Ok(())
    }

    async fn response_to_client(stream: &DuplexPipeStream<Bytes>, res: IpcResponse) -> Result<()> {
        let message = serde_json::to_vec(&res)?;

        let mut writer = BufWriter::new(stream);
        writer.write_all(&message).await?;
        writer.write_all(&[0x17]).await?;
        writer.flush().await?;
        Ok(())
    }

    pub async fn send(message: Vec<String>) -> Result<()> {
        let stream = DuplexPipeStream::connect_by_path(Self::PATH).await?;
        let data = serde_json::to_vec(&message)?;

        {
            let mut writer = BufWriter::new(&stream);
            writer.write_all(&data).await?;
            writer.write_all(&[0x17]).await?;
            writer.flush().await?;
        }

        {
            let mut reader = BufReader::new(&stream);
            let mut data = Vec::new();
            reader.read_until(0x17, &mut data).await?; // Read until end of transmission block
            data.pop(); // Remove end of transmission block

            let response: IpcResponse = serde_json::from_slice(&data)?;
            response.ok()
        }
    }
}
