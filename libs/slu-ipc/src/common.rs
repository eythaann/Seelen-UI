use std::{
    io::{BufRead, Write},
    time::Duration,
};

use interprocess::local_socket::{
    GenericFilePath, ListenerOptions, Name, ToFsName,
    traits::{Stream as _, StreamCommon as _, tokio::Stream as _},
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

use crate::{
    AsyncIpcListener, AsyncIpcStream, IpcStream, Platform, domain::IpcSystem, error::Result,
    messages::IpcResponse,
};

/// End of transmission block marker for IPC messages
pub const END_OF_TRANSMISSION_BLOCK: u8 = 0x17;

/// Timeout for IPC operations
pub const IPC_TIMEOUT: Duration = Duration::from_secs(3);

/// Maximum number of retries for failed IPC operations
pub const MAX_RETRIES: u32 = 3;

/// IPC trait for common connection operations
pub trait IPC {
    fn path() -> String;

    #[allow(async_fn_in_trait)]
    async fn server_process_id() -> Result<u32> {
        let stream = connect_async(&Self::path()).await?;
        let pid = stream
            .peer_creds()?
            .pid()
            .ok_or_else(|| std::io::Error::other("The OS did not report the server process id"))?;
        write_to_ipc_stream(&stream, &[]).await?;
        Ok(pid as u32)
    }

    fn can_stablish_connection() -> bool {
        // Run the blocking IPC probe in a dedicated thread with a hard timeout so
        // the caller never hangs if the server accepts the connection but stops
        // responding (which would leave `read_until` blocked forever).
        let path = Self::path();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let ok = (|| -> crate::error::Result<()> {
                let stream = connect(&path)?;
                send_to_ipc_stream_blocking(&stream, &[])?.ok()
            })()
            .is_ok();
            let _ = tx.send(ok);
        });
        rx.recv_timeout(IPC_TIMEOUT).unwrap_or(false)
    }
}

/// Connects to the endpoint at `path` (see [`IPC::path`]).
pub fn connect(path: &str) -> Result<IpcStream> {
    Ok(IpcStream::connect(endpoint_name(path)?)?)
}

/// Async version of [`connect`].
pub async fn connect_async(path: &str) -> Result<AsyncIpcStream> {
    Ok(AsyncIpcStream::connect(endpoint_name(path)?).await?)
}

/// Listens on the endpoint at `path`, restricted to the users allowed by the OS layer.
/// Must be called inside the tokio runtime that will drive it.
pub(crate) fn listen(path: &str) -> Result<AsyncIpcListener> {
    let options = ListenerOptions::new().name(endpoint_name(path)?);
    Ok(Platform::secure_listener(options)?.create_tokio()?)
}

fn endpoint_name(path: &str) -> Result<Name<'_>> {
    Ok(path.to_fs_name::<GenericFilePath>()?)
}

/// Reads data from an async IPC stream with timeout
pub async fn read_from_ipc_stream(stream: &AsyncIpcStream) -> Result<Vec<u8>> {
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
pub async fn write_to_ipc_stream(stream: &AsyncIpcStream, buf: &[u8]) -> Result<()> {
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
pub async fn send_to_ipc_stream(stream: &AsyncIpcStream, buf: &[u8]) -> Result<IpcResponse> {
    write_to_ipc_stream(stream, buf).await?;
    let buf = read_from_ipc_stream(stream).await?;
    IpcResponse::from_bytes(&buf)
}

/// Blocking version to test connections without needed of tokio runtime.
/// The exchange is cancelled after `IPC_TIMEOUT`, as callers like the hook dll run on threads
/// of other processes (explorer.exe tray thread) that must never hang waiting for us.
pub fn send_to_ipc_stream_blocking(stream: &IpcStream, buf: &[u8]) -> Result<IpcResponse> {
    Platform::with_io_timeout(stream, IPC_TIMEOUT, || exchange_blocking(stream, buf))
}

fn exchange_blocking(stream: &IpcStream, buf: &[u8]) -> Result<IpcResponse> {
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
