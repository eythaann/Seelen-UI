//! OS abstraction layer.
//!
//! Each submodule exposes in its `mod.rs` the OS-agnostic API used by the rest of the app, plus
//! the logic that doesn't depend on the OS. Everything native lives in a per-OS module
//! (`windows`, in the future `linux`, ...) aliased as `imp`:
//!
//! - The public API is the contract: it only forwards to `imp`, so a new OS has to provide the
//!   same items (`NativeWindow`, `NativeMonitor`, free functions, ...) and the compiler lists
//!   whatever is missing.
//! - Concepts that only exist on one OS are exposed through an extension trait
//!   (`WindowsWindowExt`, ...), only to be used from code specific to that OS.

pub mod cursor;
pub mod hook;
pub mod monitor;
pub mod paths;
pub mod process;
pub mod startup;
pub mod window;
