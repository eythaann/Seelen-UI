pub mod app;
pub mod commands;
pub mod common;
mod domain;
pub mod error;
pub mod messages;
pub mod service;
pub mod taskbar_guard;
#[cfg(windows)]
mod windows;

// Re-export main types for convenience
pub use app::AppIpc;
pub use common::IPC;
pub use service::ServiceIpc;

#[cfg(windows)]
type Platform = windows::WindowsIpc;

/// Blocking stream of the transport (`interprocess` local socket).
pub type IpcStream = interprocess::local_socket::Stream;
/// Async (tokio) stream of the transport (`interprocess` local socket).
pub type AsyncIpcStream = interprocess::local_socket::tokio::Stream;
type AsyncIpcListener = interprocess::local_socket::tokio::Listener;
