# v3 TODO — Operating System Abstraction

Goal: adding a new OS (Linux) should be "create `application/linux.rs` per module + implement the traits", without
touching `infrastructure` or `domain`. `src/hook_dll` is out of scope (Windows-only).

## Architecture rules (end state)

| Layer                               | Can                                                                                    | Cannot                                                                     |
| ----------------------------------- | -------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| `domain`                            | pure types, pure logic, **system traits** (`trait XxxSystem`)                          | import `windows`, `winreg`, `windows_api`, `HWND`, anything `cfg(windows)` |
| `application/mod.rs`                | singleton manager, `event_manager!`, state, use the trait via the `Platform` alias     | call OS APIs directly                                                      |
| `application/windows.rs`            | `impl XxxSystem for WindowsXxx`, Win32/WinRT/COM, registry, Win32 → domain conversions | depend on Tauri                                                            |
| `infrastructure.rs` / `handlers.rs` | `Handlers::*`, `emit_to_webviews`, internal events → webviews bridge                   | any import of `windows*`, `windows_api`, `unsafe`                          |

### Per-module pattern

    src/background/modules/<module>/
      mod.rs
      domain.rs            # (or domain/) types + <Module>System trait
      application/
        mod.rs             # generic Manager + `type Platform = ...` selected by cfg
        windows.rs         # #[cfg(windows)] impl <Module>System for Windows<Module>
      infrastructure.rs    # OS-agnostic

- Static dispatch, no `dyn`: in `application/mod.rs` `#[cfg(windows)] mod windows;` +
  `#[cfg(windows)] type Platform = windows::Windows<Module>;`
- The trait exposes **queries/actions** (`fn get_status() -> Result<T>`, `fn lock() -> Result<()>`) and a
  **subscription** method that receives a callback/`Sender` with domain events
  (`fn listen(on_event: impl Fn(<Module>SysEvent))`). The manager translates those into its internal `event_manager!`;
  the OS layer never knows about the manager.
- Resources with `Drop` (WinRT tokens, `HPOWERNOTIFY`, etc.) live in the `Windows<Module>` struct, which the manager
  stores as a `platform: Platform` field.
- If `windows.rs` grows too large it becomes `application/windows/mod.rs` + submodules.
- Name clash: `modules/apps/application/windows.rs` (user windows) gets renamed to `app_windows.rs` before creating the
  OS `windows.rs`.

Reference for item order and style: `src/background/modules/notifications/wpn_service.rs` and AGENTS.md.

### Shared OS primitives (`src/background/platform/`)

Two levels of abstraction coexist:

- **`platform/`**: OS primitives used across many modules/widgets (windows of other apps, monitors, well known paths,
  startup entry, global hooks). Consumed by everyone, owned by no module.
- **`modules/<module>/application/windows.rs`**: OS code that only one module needs (power, audio, bluetooth, ...),
  following the per-module pattern above.

Rule of thumb: if two or more modules/widgets need it, it goes to `platform/`; otherwise it stays in the module.

Layout of each `platform/<primitive>/`:

    platform/<primitive>/
      mod.rs       # OS-agnostic public API (the contract) + logic that doesn't depend on the OS
      windows.rs   # #[cfg(windows)] native impl, aliased as `imp` (a folder when it needs submodules)

- No contract traits (same approach as `std::sys`): the public API only forwards to `imp`, so a new OS provides the same
  items (`NativeWindow`, `NativeMonitor`, `NativeProcess`, free functions) and the compiler lists what's missing.
- Handle-like types are newtypes over the native one (`struct Window(imp::NativeWindow)`), static dispatch. Logic that
  only composes other methods (snapshots, drag tracking, stable id helpers) lives in `mod.rs`, not in `imp`.
- Windows-only extras are exposed through an extension trait (`WindowsWindowExt`, `WindowsMonitorExt`,
  `WindowsProcessExt`) that only Windows specific code may import.
- Stateless APIs (`paths`, `startup`, `cursor`) are free functions forwarding to `imp`.
- `platform/*/windows*` may use `windows_api/` as plumbing; nothing outside Windows impls may.
- Event vocabularies (`AppWindowEvent`) are still the native ones; making them agnostic is pending (see below).

---

## Phase 0.5 — Wire `platform/` into the app

- [x] `hook` — global win event hook (old `src/background/hook.rs`) and cursor tracking moved to `platform/hook` and
      `platform/cursor`, started from `app.rs`. `AppWindowEvent` carries the `platform::window::Window`.
- [x] `paths` — consumers migrated (`logger.rs`, `cli/svc_pipe.rs`, `modules/start`, `utils/constants.rs`,
      `utils/pwsh.rs`). Remaining `WindowsApi::known_folder` calls are Windows impl code
      (`notifications/wpn_service.rs`, `platform/window/windows` for `APP_FRAME_HOST_PATH`) and move with their owners.
- [x] `startup` — `SeelenUI::is_auto_start_enabled` and `ServicePipe::start_service` use `platform::startup`; the
      duplicated Task Scheduler code was removed.
- [x] `monitor` + `window` — every consumer outside `windows_api/` uses `platform::{monitor::Monitor, window::Window}`
      (`Window::enumerate`/`Monitor::enumerate` replace `WindowEnumerator`/`MonitorEnumerator`). The
      `#![allow(dead_code)]` of `platform/mod.rs` was removed.
  - `get_app_config`/`slu_category` are app-config business logic, not OS: they left the native contract and live in
    `state/application/apps_config.rs` (via `exe_path()`).
  - `WindowsWindowExt` is still imported where the code is Win32 by nature (`hwnd()` for styles/z-order/shadow, UMID,
    UWP frames): `modules/apps/application/{windows,previews}.rs`, `clipboard`, `widgets/weg/handler.rs`,
    `widgets/window_manager/{mod,hook,handler,state_v2}.rs`. Those get resolved in Phase 1/2 when each owner gets its
    trait (candidates to promote to `platform::window`: `close`, z-order, frame bounds without shadow).
  - `widgets/wallpaper_manager` still uses `WindowEnumerator` for the `WorkerW` lookup: Win32 plumbing, moves with it.
- [x] `windows_api/{window,monitor,process}` removed: their Win32 implementation now lives in
      `platform/{window,monitor,process}/windows*` (monitor split in `display_config.rs` / `display_view.rs`),
      `WinEvent` in `platform/hook/windows/event.rs`, and the DDC/CI code (only used by brightness) in
      `modules/monitors/brightness/ddcci.rs`. `WindowEnumerator`/`MonitorEnumerator` yield native handles.
  - [x] `process` — `platform::process::Process` (`exe_path`, `exe_name`, `display_name`, `is_frozen`, `is_seelen`,
        `snapshot`), `Window::process()` is part of the agnostic API; handles/AppX are behind `WindowsProcessExt`.
  - [x] Flattened: one native layer per primitive, no contract traits nor forwarding wrappers.
- [ ] `AppWindowEvent`: replace the native (`WinEvent`) vocabulary with an OS-agnostic enum of domain events.
- [ ] `platform/window/mod.rs` still depends on modules (`is_interactable_window` from `modules/apps`, `VdManager` for
      `workspace_id`), inherited from the old `windows_api/window`. `platform/` must not depend on its consumers: move
      `is_interactable_and_not_hidden`/`workspace_id` to their owners (or invert the dependency).

---

## Phase 0 — Foundations

- [ ] Document the new pattern in `AGENTS.md` ("Backend: System Modules" section) and replace the current example.
- [ ] `src/background/error.rs` / `src/service/error.rs`: put the `Windows(windows::core::Error)` variant and
      `WindowsResultExt` behind `#[cfg(windows)]`.
- [ ] Make `windows_api/` explicitly Windows-impl plumbing (`#[cfg(windows)] mod windows_api;`). It may only be used
      from `application/windows.rs` (and equivalents outside `modules/`).
- [ ] `modules/mod.rs`: keep `event_manager!` OS-agnostic (verify it has no Windows dependency).
- [ ] `windows_api/event_window.rs::subscribe_to_background_window` is the source of `WM_*` messages; it stays an
      internal detail of the Windows impls.
- [ ] Add a CI/lint check (grep) that fails if `infrastructure.rs`, `handlers.rs` or `domain*` import `windows`,
      `windows_core`, `winreg`, `windows_api`, `win_screenshot`.

## Phase 1 — System modules (`src/background/modules/`)

For each module: (1) define the `trait` in domain, (2) move Win32 logic + Win32→domain conversions to
`application/windows.rs`, (3) leave `application/mod.rs` generic, (4) clean up `infrastructure`.

Start with the small ones to validate the pattern:

- [ ] **power** (pilot)
  - infra calls `WindowsApi::exit_windows/set_suspend_state/lock_machine/initiate_shutdown` and uses `SHUTDOWN_*` →
    trait
    `PowerSystem { log_out, suspend, hibernate, lock, restart(install_updates), shutdown(install_updates),
    has_pending_os_updates, power_status, power_mode, batteries, listen }`.
  - `domain.rs` uses `SYSTEM_POWER_STATUS`/`EFFECTIVE_POWER_MODE` → move conversions to `windows.rs`.
  - registry `PENDING_UPDATES_REGISTRY_KEYS` → `windows.rs`.
- [ ] **media/devices** — infra: `WindowsApi::set_default_audio_device`; domain: `Win32::Media::Audio` types.
- [ ] **media/players** — domain uses WinRT types (GSMTC).
- [ ] **media/waveform** — domain uses `IAudioCaptureClient/IAudioClient`.
- [ ] **clipboard** — infra uses `windows_api::input::Keyboard` (paste); review `persistence.rs` (DPAPI?).
- [ ] **monitors** (+ `brightness/`) — infra uses `MonitorEnumerator`; domain has 1 Win32 ref.
- [ ] **network** — infra uses `NetworkListManager`/`Com` + `unsafe GetConnectivity`; `domain/` uses `IpHelper`.
- [ ] **notifications** — infra uses `Com`, `AppUserModelId`, `WindowsString`, `WindowsApi::execute`; `wpn_service.rs`
      moves to `application/windows/`.
- [ ] **apps** — infra uses `IShellDispatch6::ToggleDesktop`, `Mouse`, `Window`; rename `application/windows.rs` →
      `app_windows.rs`; `flashing`, `previews`, `msix*`, `badges` are Windows → impl.
- [ ] **radios** (`bluetooth/`, `wifi/`) — `bluetooth/handlers.rs` uses `DevicePairingResultStatus`; rename
      `manager.rs`/`handlers.rs` to the standard layout.
- [ ] **focus_assist**, **fonts**, **trash_bin**, **user** (+ `indexing/`), **start** — only application has Win32:
      extract trait + `windows.rs`.
- [ ] **system_settings** (+ `language/`, `nightlight/`) — delete `language/application.rs.old`.
- [ ] **system_tray** — `tray_hook_loader`, `tray_icon`, `util` are 100% Windows → `application/windows/`.
- [ ] **system** — `gpu.rs` (PDH/DXGI) to `application/windows.rs`; `tauri.rs` → `infrastructure.rs`;
      `IS_INTERACTIVE_SESSION` behind the trait.

## Phase 2 — Rest of the main app (`src/background/`, outside `modules/`)

Same criteria; these areas don't follow the module pattern yet. Inventory (files with Win32 / total):

- [ ] `session/` (1/4) — already layered, clean up.
- [ ] `state/` (3/8) — infra + application.
- [ ] `virtual_desktops/` (3/6) — move to domain/application/infrastructure layout.
- [ ] `widgets/` (17/27) — `window_manager`, `weg`, `toolbar`, `task_switcher`, `popups`, `wallpaper_manager`: put
      positioning/HWND behind traits (windows, monitors, app bar).
- [ ] `utils/` (10/16) — split generic utilities from Win32 ones (Win32 ones go to `windows_api/`).
- [ ] `cli/` (1/7) — no direct calls.
- [x] ~~`hook.rs` (global win event hook)~~ — moved to `platform/hook/` (see Phase 0.5).
- [ ] `app.rs`, `main.rs`, `exposed.rs`, `logger.rs`, `telemetry.rs` — isolate Win32 code in `#[cfg(windows)]` functions
      or in a `platform` module.

## Phase 3 — Service (`src/service/`)

Currently flat (`power.rs`, `shutdown.rs`, `task_scheduler.rs`, `enviroment.rs`, `hotkeys.rs`, `app_management.rs`) and
`cli/processing.rs` (the infrastructure equivalent) calls `WindowsApi::*` and `TaskSchedulerHelper` directly.

- [ ] Reorganize into `src/service/modules/<module>/{domain.rs, application/{mod.rs,windows.rs}, infrastructure.rs}`:
  - `power` (suspend/resume listener), `startup` (task_scheduler), `environment` (PATH via registry, fix the
    `enviroment` typo), `taskbar` (shutdown.rs: hide/restore native taskbar + shutdown listener), `hotkeys`,
    `windows_positioning` (ShowWindow/SetPosition/SetForeground/DeferWindowPositions), `app_management`.
- [ ] `cli/processing.rs` becomes OS-agnostic infrastructure: it only dispatches `SvcAction` to managers/traits.
- [ ] `string_utils.rs` (`WindowsString`) and `windows_api/` → `#[cfg(windows)]`, only used from Windows impls.

## Phase 4 — Crates and conditional compilation

- [ ] `src/Cargo.toml`: move `windows`, `windows-core`, `windows-collections`, `windows-future`, `winreg`,
      `win-screenshot`, `win-hotkeys` to `[target.'cfg(windows)'.dependencies]`.
- [ ] `libs/positioning` (9 Win32 files) and `libs/slu-ipc` (5): same trait + Windows impl pattern, or gate them.
- [ ] `libs/slu-ipc` `SvcAction` uses `hwnd`: introduce an opaque OS-agnostic `WindowId`.
- [ ] `libs/utils` (1 ref) review.
- [ ] Verify `cargo check` still passes on Windows and, as a goal, that
      `cargo check --target
      x86_64-unknown-linux-gnu -p seelen-ui` fails only with "trait not implemented" errors
      (optional empty `application/linux.rs` / `unimplemented!()` stub).

## Per-module verification

- `cargo check` (never `--release`).
- Control grep:
  `rg "windows::|windows_core|winreg|windows_api|HWND" src/background/modules -g "infrastructure.rs"
  -g "handlers.rs" -g "domain*"`
  → empty.
- `deno task build:rs` in `libs/core` if any exposed type changes (it shouldn't: this is an internal refactor).
- Manual testing by the user in the app (power actions, audio devices, notifications, etc.).
