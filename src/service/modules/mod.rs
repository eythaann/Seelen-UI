//! OS-agnostic service modules (v3 architecture).
//!
//! Each module follows the same layout as the main app `system_modules`:
//!
//! ```text
//! <module>/
//!   mod.rs               # public API: re-exports the manager (and the domain types it needs)
//!   domain.rs            # pure types/logic + `<Module>System` trait (the OS contract)
//!   application/
//!     mod.rs             # singleton manager, generic over `Platform`
//!     windows.rs         # #[cfg(windows)] `impl <Module>System for Windows<Module>`
//! ```
//!
//! - The infrastructure of the service is `cli/processing.rs` (it dispatches the `SvcAction`s
//!   received through IPC to the managers) and `main.rs` (startup/shutdown orchestration).
//! - `application` and `domain` are private; `mod.rs` never re-exports native types.
//! - `domain` never imports anything OS specific.
//! - `application/mod.rs` only talks to the OS through the trait, via the `Platform` alias
//!   selected by `cfg` (static dispatch, no `dyn`). A new OS adds `application/<os>.rs` plus its
//!   alias, and the compiler lists whatever is missing.
//! - Singletons split creation from initialization: `create()` never fails (no OS calls) and
//!   `init()` returns `Result`; a failed init leaves a degraded but usable manager, never a panic.
//! - The native layer never knows about the manager: it reports through the callback it receives.

pub mod app_management;
pub mod environment;
pub mod hotkeys;
pub mod lifecycle;
pub mod native_wallpaper;
pub mod startup;
pub mod taskbar;
pub mod window_positioning;
