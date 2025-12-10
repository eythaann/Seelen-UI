use std::{
    io::{BufRead, Write},
    time::Duration,
};

use interprocess::os::windows::{
    named_pipe::{
        DuplexPipeStream, pipe_mode::Bytes, tokio::DuplexPipeStream as AsyncDuplexPipeStream,
    },
    security_descriptor::{AsSecurityDescriptorMutExt, SecurityDescriptor},
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

use crate::{error::Result, messages::IpcResponse};

/// https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-control
pub static SE_DACL_PROTECTED: u16 = 4096u16;

/// End of transmission block marker for IPC messages
pub const END_OF_TRANSMISSION_BLOCK: u8 = 0x17;

/// Timeout for IPC operations
pub const IPC_TIMEOUT: Duration = Duration::from_secs(3);

/// Maximum number of retries for failed IPC operations
pub const MAX_RETRIES: u32 = 3;

/// IPC trait for common connection operations
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
        let response = send_to_ipc_stream_blocking(&stream, &[])?;
        response.ok()
    }

    fn can_stablish_connection() -> bool {
        Self::test_connection().is_ok()
    }
}

/// Creates a security descriptor for IPC pipes
pub fn create_security_descriptor() -> Result<SecurityDescriptor> {
    let mut sd = SecurityDescriptor::new()?;
    unsafe { sd.set_dacl(std::ptr::null_mut(), false)? };
    sd.set_control(SE_DACL_PROTECTED, SE_DACL_PROTECTED)?;
    Ok(sd)
}

/// Reads data from an async IPC stream with timeout
pub async fn read_from_ipc_stream(stream: &AsyncDuplexPipeStream<Bytes>) -> Result<Vec<u8>> {
    let mut reader = BufReader::new(stream);
    let mut buf = Vec::new();

    tokio::time::timeout(IPC_TIMEOUT, async {
        reader.read_until(END_OF_TRANSMISSION_BLOCK, &mut buf).await
    })
    .await
    .map_err(|_| crate::error::Error::Timeout("Failed to read from IPC stream".to_string()))??;

    buf.pop();
    Ok(buf)
}

/// Writes data to an async IPC stream with timeout
pub async fn write_to_ipc_stream(stream: &AsyncDuplexPipeStream<Bytes>, buf: &[u8]) -> Result<()> {
    let mut writter = BufWriter::new(stream);

    tokio::time::timeout(IPC_TIMEOUT, async {
        writter.write_all(buf).await?;
        writter.write_all(&[END_OF_TRANSMISSION_BLOCK]).await?;
        writter.flush().await?;
        Ok::<(), std::io::Error>(())
    })
    .await
    .map_err(|_| crate::error::Error::Timeout("Failed to write to IPC stream".to_string()))??;

    Ok(())
}

/// Sends data and receives response from an async IPC stream
pub async fn send_to_ipc_stream(
    stream: &AsyncDuplexPipeStream<Bytes>,
    buf: &[u8],
) -> Result<IpcResponse> {
    write_to_ipc_stream(stream, buf).await?;
    let buf = read_from_ipc_stream(stream).await?;
    IpcResponse::from_bytes(&buf)
}

/// Blocking version to test connections without needed of tokio runtime
pub fn send_to_ipc_stream_blocking(
    stream: &DuplexPipeStream<Bytes>,
    buf: &[u8],
) -> Result<IpcResponse> {
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

/// Sends data with retry logic and exponential backoff
pub async fn send_with_retry<F, Fut>(send_fn: F) -> Result<()>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<IpcResponse>>,
{
    let mut last_error = None;

    for attempt in 0..MAX_RETRIES {
        match send_fn().await {
            Ok(response) => return response.ok(),
            Err(err) => {
                last_error = Some(err);

                if attempt < MAX_RETRIES - 1 {
                    // Exponential backoff: 100ms, 200ms, 400ms
                    let delay = Duration::from_millis(100 * 2u64.pow(attempt));
                    log::debug!(
                        "IPC send failed (attempt {}/{}), retrying in {}ms",
                        attempt + 1,
                        MAX_RETRIES,
                        delay.as_millis()
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        crate::error::Error::Timeout("Unknown error during IPC send".to_string())
    }))
}
