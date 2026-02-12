pub mod app;
pub mod common;
pub mod error;
pub mod messages;
pub mod service;

// Re-export main types for convenience
pub use app::AppIpc;
pub use common::IPC;
pub use service::ServiceIpc;
