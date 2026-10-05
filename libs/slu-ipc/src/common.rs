use std::{
    io::{BufRead, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::RecvTimeoutError,
    },
    time::Duration,
};

use interprocess::os::windows::named_pipe::{
    DuplexPipeStream, pipe_mode::Bytes, tokio::DuplexPipeStream as AsyncDuplexPipeStream,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::{
        IO::CancelSynchronousIo,
        Threading::{GetCurrentThreadId, OpenThread, THREAD_TERMINATE},
    },
};

use crate::{error::Result, messages::IpcResponse};

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
        let stream = AsyncDuplexPipeStream::connect_by_path(Self::path()).await?;
        let pid = stream.server_process_id()?;
        write_to_ipc_stream(&stream, &[]).await?;
        Ok(pid)
    }

    fn can_stablish_connection() -> bool {
        // Run the blocking IPC probe in a dedicated thread with a hard timeout so
        // the caller never hangs if the server accepts the connection but stops
        // responding (which would leave `read_until` blocked forever).
        let path = Self::path();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let ok = (|| -> crate::error::Result<()> {
                let stream = DuplexPipeStream::<Bytes>::connect_by_path(path)?;
                send_to_ipc_stream_blocking(&stream, &[])?.ok()
            })()
            .is_ok();
            let _ = tx.send(ok);
        });
        rx.recv_timeout(IPC_TIMEOUT).unwrap_or(false)
    }
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

/// Blocking version to test connections without needed of tokio runtime.
/// The exchange is cancelled after `IPC_TIMEOUT`, as callers like the hook dll run on threads
/// of other processes (explorer.exe tray thread) that must never hang waiting for us.
pub fn send_to_ipc_stream_blocking(
    stream: &DuplexPipeStream<Bytes>,
    buf: &[u8],
) -> Result<IpcResponse> {
    // CancelSynchronousIo needs a real handle with THREAD_TERMINATE access, not the pseudo one
    let thread = unsafe { OpenThread(THREAD_TERMINATE, false, GetCurrentThreadId())? };
    let thread_addr = thread.0 as isize; // HANDLE is not Send
    let timed_out = AtomicBool::new(false);

    let result = std::thread::scope(|scope| {
        let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
        let timed_out = &timed_out;
        scope.spawn(move || {
            if let Err(RecvTimeoutError::Timeout) = done_rx.recv_timeout(IPC_TIMEOUT) {
                timed_out.store(true, Ordering::Release);
                // retry until the exchange finishes, the I/O could start right after a cancel
                while let Err(RecvTimeoutError::Timeout) =
                    done_rx.recv_timeout(Duration::from_millis(10))
                {
                    let _ = unsafe { CancelSynchronousIo(HANDLE(thread_addr as _)) };
                }
            }
        });
        let result = exchange_blocking(stream, buf);
        drop(done_tx);
        result
    });

    let _ = unsafe { CloseHandle(thread) };
    if timed_out.load(Ordering::Acquire) && result.is_err() {
        return Err(crate::error::Error::Timeout(
            "No response from IPC stream".to_string(),
        ));
    }
    result
}

fn exchange_blocking(stream: &DuplexPipeStream<Bytes>, buf: &[u8]) -> Result<IpcResponse> {
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
