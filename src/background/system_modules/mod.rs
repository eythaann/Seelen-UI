//! OS-agnostic system modules (v3 architecture).
//!
//! Each module follows the same layout:
//!
//! ```text
//! <module>/
//!   mod.rs               # infrastructure: `Handlers::*` + internal events -> webviews bridge
//!   domain.rs            # pure types + `<Module>System` trait (the OS contract)
//!   application/
//!     mod.rs             # singleton manager + `event_manager!`, generic over `Platform`
//!     windows.rs         # #[cfg(windows)] `impl <Module>System for Windows<Module>`
//! ```
//!
//! - The module's `mod.rs` is the infrastructure layer. `application` and `domain` are private
//!   (`mod`, never `pub mod`): the only thing a module exposes is its `impl Handlers` block.
//!   Exception: when other parts of the crate consume the manager (lookups, subscriptions), the
//!   `mod.rs` re-exports just that (`pub use self::application::XManager;`), never the layers.
//! - `domain` never imports anything OS specific.
//! - `application/mod.rs` only talks to the OS through the trait, via the `Platform` alias
//!   selected by `cfg` (static dispatch, no `dyn`). A new OS adds `application/<os>.rs` plus its
//!   alias, and the compiler lists whatever is missing.
//! - The native struct owns the OS resources (WinRT tokens, handles, ...) and releases them on
//!   `Drop`; the manager keeps it as its `platform` field.
//! - Singletons split creation from initialization: `create()` never fails (no OS calls) and
//!   `init()` returns `Result`. The `LazyLock` runs `create()` + `init().log_error()`, never a
//!   panic, so a failed init leaves a degraded but usable manager instead of a poisoned one.
//!   A heavy initial load awaited at startup (e.g. `start`) goes in a separate `initialize()`:
//!   `init()` only sets up the listeners, so `instance()` stays cheap and lookups never block.
//! - The native layer never knows about the manager: it reports changes through the callback
//!   received in `<Module>System::init`.
//! - The infrastructure (`mod.rs`) is OS-agnostic: no `windows*`, `windows_api` nor `unsafe`.
//!
//! - Native facade variant (`system_tray`, `radios`, `wifi`, `bluetooth`, `media_devices`): when the
//!   state is a web of OS objects and callbacks, the native layer keeps it in its own singleton
//!   (`<Module>State`, with an internal `event_manager!` to get the OS callbacks out of the OS
//!   threads) and the native struct is a facade over it. The manager stays thin.
//! - A module may be split in several when it has several managers (e.g. `monitors` +
//!   `brightness`, `apps` + `app_previews`): each one keeps this same layout.

pub mod app_previews;
pub mod apps;
pub mod bluetooth;
pub mod brightness;
pub mod clipboard;
pub mod focus_assist;
pub mod fonts;
pub mod languages;
pub mod media_devices;
pub mod media_players;
pub mod media_waveform;
pub mod monitors;
pub mod network;
pub mod notifications;
pub mod power;
pub mod radios;
pub mod start;
pub mod system_info;
pub mod system_settings;
pub mod system_tray;
pub mod trash_bin;
pub mod user;
pub mod user_folders;
pub mod wifi;
