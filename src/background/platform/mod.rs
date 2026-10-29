//! OS abstraction layer.
//!
//! Every submodule exposes an OS-agnostic API in its `mod.rs` and declares there the contract
//! (a private trait) that each OS has to implement in its own file (`windows.rs`, in the future
//! `linux.rs`, ...). The rest of the app must only consume the agnostic API.

// Not wired into the app yet: the current code still uses `windows_api` directly.
#![allow(dead_code, unused_imports)]

pub mod hook;
pub mod monitor;
pub mod paths;
pub mod startup;
pub mod window;
