# v3 TODO — Operating System Abstraction

Goal: adding a new OS (Linux) should be "create `application/linux.rs` per module + implement the traits", without
touching `infrastructure` or `domain`. `src/hook_dll` is out of scope (Windows-only).

## Architecture rules (end state)

| Layer                              | Can                                                                                    | Cannot                                                                     |
| ---------------------------------- | -------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| `domain`                           | pure types, pure logic, **system traits** (`trait XxxSystem`)                          | import `windows`, `winreg`, `windows_api`, `HWND`, anything `cfg(windows)` |
| `application/mod.rs`               | singleton manager, `event_manager!`, state, use the trait via the `Platform` alias     | call OS APIs directly                                                      |
| `application/windows.rs`           | `impl XxxSystem for WindowsXxx`, Win32/WinRT/COM, registry, Win32 → domain conversions | depend on Tauri                                                            |
| `<module>/mod.rs` (infrastructure) | `Handlers::*`, `emit_to_webviews`, internal events → webviews bridge                   | any import of `windows*`, `windows_api`, `unsafe`                          |

### Per-module pattern

Migrated modules move from `src/background/modules/` to `src/background/system_modules/`, one at a time (layout
documented in `system_modules/mod.rs`). Reference implementation: `system_modules/focus_assist`.

    src/background/system_modules/<module>/
      mod.rs               # infrastructure (OS-agnostic): Handlers + events -> webviews
      domain.rs            # (or domain/) types + <Module>System trait
      application/
        mod.rs             # generic Manager + `type Platform = ...` selected by cfg
        windows.rs         # #[cfg(windows)] impl <Module>System for Windows<Module>

- The module's `mod.rs` **is** the infrastructure layer. `application` and `domain` are private (`mod`, never `pub
  mod`); the only thing that leaves the module is its `impl crate::tauri_handlers::Handlers` block.
- Static dispatch, no `dyn`: in `application/mod.rs` `#[cfg(windows)] mod windows;` + `#[cfg(windows)] type Platform =
  windows::Windows<Module>;`
- **Create and init are separate** (managers and native structs alike, they are singletons behind a `LazyLock`):
  - `create() -> Self` never fails: it only builds the struct, no OS calls.
  - `init(&mut self) -> Result<()>` acquires the resources and subscribes; it may fail.
  - The `LazyLock` does `let mut m = Manager::create(); m.init().log_error(); m`. Never `expect`/`unwrap`/panic there: a
    failed init leaves a degraded but usable manager (queries return `Err`/defaults) instead of poisoning the singleton
    and crashing every caller.
  - Native resources acquired in `init` are stored as `Option<_>` and queries return `Err` while they are `None`.
- The trait exposes `create`, `init` and **queries/actions** (`fn get_status() -> Result<T>`, `fn lock() ->
  Result<()>`). `init` receives the callback with the domain events (`fn init(&mut self, on_event: impl
  Fn(<Module>SysEvent))`); the manager passes `Self::send` (or translates them into its internal `event_manager!` event,
  e.g. `power`: `PowerSysEvent` → `PowerEvent`), the OS layer never knows about the manager. Native C callbacks without
  closures receive it through their context pointer.
- Resources with `Drop` (WinRT tokens, `HPOWERNOTIFY`, etc.) live in the `Windows<Module>` struct, which the manager
  stores as a `platform: Platform` field. `Drop` must handle a partially initialized struct.
- Mutable state goes in fields behind a `TracedMutex` (`info: TracedMutex<T>`), never the whole manager: `instance()`
  returns `&'static Self`. When the OS callback needs that state (cache, dedupe), the manager passes an associated fn
  (`Self::on_platform_event`) that reaches the state via `Self::instance()`.
- If `windows.rs` grows too large it becomes `application/windows/mod.rs` + submodules.
- Name clash: `modules/apps/application/windows.rs` (user windows) gets renamed to `app_windows.rs` before creating the
  OS `windows.rs`.

Reference for item order and style: `src/background/windows_api/wpn_service.rs` and AGENTS.md.

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
- Event vocabularies are OS-agnostic (`platform::hook::WindowEvent`); each impl translates its native events.

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
  - `WindowsWindowExt` is only imported from Windows impls (`apps`, `app_previews`, `clipboard`,
    `wallpaper_manager/windows.rs`). `close`, `set_z_order`, `place_below`, `is_resizable` and `is_always_on_top` were
    promoted to `platform::window` (Phase 2, `widgets/`).
- [x] `windows_api/{window,monitor,process}` removed: their Win32 implementation now lives in
      `platform/{window,monitor,process}/windows*` (monitor split in `display_config.rs` / `display_view.rs`),
      `WinEvent` in `platform/hook/windows/event.rs`, and the DDC/CI code (only used by brightness) in
      `modules/monitors/brightness/ddcci.rs`. `WindowEnumerator`/`MonitorEnumerator` yield native handles.
  - [x] `process` — `platform::process::Process` (`exe_path`, `exe_name`, `display_name`, `is_frozen`, `is_seelen`,
        `snapshot`), `Window::process()` is part of the agnostic API; handles/AppX are behind `WindowsProcessExt`.
  - [x] Flattened: one native layer per primitive, no contract traits nor forwarding wrappers.
- [x] `AppWindowEvent` is `(WindowEvent, Window)`: OS-agnostic vocabulary in `platform/hook/mod.rs`. `WinEvent` is
      private to `platform/hook/windows`, which translates it (`WinEvent::to_window_event`, unconsumed events are
      dropped) and emits the debounced/throttled `RectChanged`/`ForegroundRectChanged` itself.
- [x] `platform/window/mod.rs` no longer depends on `VdManager`: `Window::workspace_id()` was inverted into
      `VdManager::workspace_of(&Window)`.
  - Accepted exception: `platform/window/windows/snapshot.rs` uses `system_modules::start::StartMenuManager` because it
    needs info about the app that only lives in the start menu.

---

## Phase 0 — Foundations

- [x] Document the new pattern in `AGENTS.md` ("Backend: System Modules" section) and replace the current example.
- [x] `src/background/error.rs` / `src/service/error.rs`: the conversions from Windows-only crates (`windows`, `wmi`,
      `win_screenshot`, `win_hotkeys`) and `WindowsResultExt` are behind `#[cfg(windows)]` (`define_app_errors!` accepts
      attributes per variant).
- [x] `#[cfg(windows)] mod windows_api;` in both binaries. It may only be used from `application/windows.rs` (and
      equivalents outside `modules/`).
- [x] `event_manager!` (now `utils/event_manager.rs`) is OS-agnostic.
- [x] `windows_api/event_window.rs::subscribe_to_background_window` is the source of `WM_*` messages; it stays an
      internal detail of the Windows impls (only used from `application/windows*`).

## Phase 1 — System modules (`src/background/system_modules/`) ✔

For each module: (1) define the `trait` in domain, (2) move Win32 logic + Win32→domain conversions to
`application/windows.rs`, (3) leave `application/mod.rs` generic, (4) clean up `infrastructure`.

Start with the small ones to validate the pattern:

- [x] **focus_assist** (pilot) — `FocusAssistSystem { create, init, is_active, set_active }`; `WindowsFocusAssist`
      acquires `FocusSessionManager` in `init` and owns the WinRT token (`Drop`). The domain event `FocusAssistEvent`
      doubles as the `event_manager!` event: a separate internal event type is only needed when the manager adds or
      transforms events.
- [x] **trash_bin** — `TrashBinSystem { create, init, query, empty }`; `WindowsTrashBin` is stateless (background window
      subscriptions are app-lifetime, no `Drop`). The manager caches `TrashBinInfo` and drops the shell notifications
      that leave it unchanged (`TrashBinInfo` now derives `PartialEq, Eq`).

- [x] **power** — `PowerSystem { create, init, power_status, batteries, has_pending_os_updates, log_out, suspend,
      hibernate, lock, restart, shutdown }`. First module with a separate OS event (`PowerSysEvent`: only signals what
      changed, plus `Resumed`) and manager event (`PowerEvent`: carries the data); the manager queries, dedupes and
      handles the post-resume refresh. The native callbacks get the `on_event` closure through their Win32 context
      pointer (`Arc` kept in `WindowsPower`). Fixed on the way: the battery registration was released with
      `UnregisterPowerSettingNotification` (wrong API, now `PowerSettingUnregisterNotification`) and the power mode one
      was never released. `PowerStatus` (`Default, PartialEq, Eq`) and `PowerMode` (`Default`) derives added in core.
  - Done: the game mode → suspend widgets logic moved from the infra bridge to `WidgetManager::create` (subscribes to
    `PowerManager`, re-exported with `PowerEvent`). It used to be registered only on the first power command.
  - Pending: `PowerStatus` fields are still Win32 shaped (`ac_line_status: u8`, ...).
- [x] **fonts** — `FontsSystem { create, init, fonts }`. Stateless (`WindowsFonts` unit struct, DirectWrite factory per
      query, so the old `unsafe impl Send/Sync` is gone); the manager sorts the list. `FontsChanged` was never emitted
      (no OS listener): now wired to `WM_FONTCHANGE` on the background window → `FontsSysEvent::Changed` → the manager
      queries and sends `FontsEvent::Changed { fonts }`.
- [x] **start** — `StartMenuSystem { create, init, user_shortcuts, system_shortcuts, packaged_apps }`; dedup/lookups in
      the manager. `init` only sets up the listeners (folder watcher + package catalog, now with tokens removed on
      `Drop`); the heavy load stays in `initialize()` awaited at startup. Re-exports `StartMenuManager`.
- [x] **user** + **user_folders** (old `user/indexing`, now its own module) — the folder scan and ignore lists are
      domain/application; only paths + watchers are native. Fixed: `get_user` emitted `UserChanged` unredacted in
      streaming mode. `User` derives `Default` in core.
- [x] **media** → **media_devices**, **media_players**, **media_waveform**. Devices: the COM objects stay native
      (internal `event_manager!` keeps COM calls out of the callbacks); waveform: `LoopbackCapture` trait, ring
      buffer/FFT in application/domain, restarts on `MediaDevicesEvent::DefaultOutputChanged`; players: the grace
      period, thumbnail retry and seek clamping (`seek_range`) moved to the manager.
- [x] **clipboard** — history, limits, dedup and eviction in the manager; WinRT/STA, DPAPI persistence and paste are
      native (`ClipboardStore` in domain).
- [x] **monitors** + **brightness** (own module, re-exported for monitors) — the view/monitor diff and debounce are in
      the manager; `MonitorManagerEvent` now uses struct variants.
- [x] **network** — `NetworkSysEvent { AdaptersChanged, StatusChanged }` → `NetworkEvent` with data; the IP change
      callback gets `on_event` through its context pointer.
- [x] **notifications** — the list lives in the manager; WinRT listener/polling, toast parsing and activation are native
      (`activation.rs`, `content.rs`). `wpn_service.rs` → `windows_api/wpn_service.rs` (shared with apps).
- [x] **apps** + **app_previews** (own module) — the windows list and the attention/badge rules live in the manager; the
      hook handling reports `AppsSysEvent::{Added, Removed, Changed { change: WindowChange }}`. `is_interactable_window`
      → `platform/window/windows/interactable.rs`; `msix` → `windows_api/msix/`; `WindowBadges` →
      `windows_api/window_badges.rs` (own event, no longer touches the apps manager).
- [x] **system_info** (old `system`) — `sysinfo` polling and diffing in the manager, only the GPU (PDH) is native.
- [x] **system_tray** — all native (hook injection, registry, IPC icon events) behind `TrayState` (`Arc` + `Weak` self
      ref for the registry lookup threads); the manager only forwards.
- [x] **system_settings** + **languages** (own module) — accent palette generation in domain; settings index and night
      light are native. The Bond codec moved to `utils/bond` (generic, referenced by `AppError`).
      `language/application.rs.old` deleted.
- [x] **radios**, **wifi**, **bluetooth** (one module each) — "native facade" variant: the old managers became the
      native singletons (`RadiosState`, `WifiState`, `BluetoothState`) with their internal `event_manager!`; the facades
      implement the traits. Kicking the WiFi scan / BT discovery when a radio is turned on moved to the radios manager.
- [x] `modules/` removed; `event_manager!` → `utils/event_manager.rs`.
  - Pending: the "native facade" modules (system_tray, radios, wifi, bluetooth, media_devices) still keep their state in
    the native layer: lift the OS-agnostic parts when a second OS is added.
  - Done: the interactive OS session flag lives in `platform::session::is_interactive()` (OS user session, not
    `crate::session`, which is the Seelen UI one); the per-module `is_session_interactive` trait methods were removed.

## Phase 2 — Rest of the main app (`src/background/`, outside `modules/`)

Same criteria; these areas don't follow the module pattern yet. Inventory (files with Win32 / total):

- [x] `session/` — module layout (`mod.rs` infra re-exporting `SessionManager`/`SessionManagerEvent` and
      `reemit_session`). The only OS dependency, the token storage, is the `CredentialStore` trait (domain) implemented
      by `WindowsCredentials` (`PasswordVault`). `read_credential` is no longer public.
- [x] `state/` — module layout: `mod.rs` infra (handlers + webviews bridge, re-exports `FULL_STATE`, `AppSettings`,
      `PERFORMANCE_MODE`, `TOOLBAR_ITEMS_MANAGER`, `WEG_ITEMS_MANAGER`, `StateManager`, `StateEvent`; consumers use
      `crate::state::*`), `domain.rs` (`StateEvent { SettingsChanged, PerformanceModeChanged }`), `application/` private
      and without Tauri. `StateManager` (unit struct, `FULL_STATE` is swapped as a whole) emits the events; performance
      mode and workspace wallpapers subscribe to it (and to `PowerManager`) instead of listening to Tauri events. Native
      wallpaper and AUMID → exe moved to `platform::shell::{wallpaper, set_wallpaper, app_executable}`; the resources
      watcher uses `RecommendedWatcher`.
  - Done: the first-run import of the native taskbar pins goes through `SeelenWeg::native_pinned_apps()`.
- [x] `virtual_desktops/` — no OS code left (everything goes through `platform/`), so no `windows.rs`. Module layout:
      `mod.rs` infra (handlers + webviews bridge, re-exports `VdManager`, `MINIMIZED_BY_WORKSPACES`,
      `VirtualDesktopEvent`, `WorkspaceWallpapersManager`), `cli.rs`, `domain.rs` (`VirtualDesktopEvent` +
      `get_indexes`), `application/{mod.rs, storage.rs (old bridge.rs), wallpapers.rs}`.
- [x] `widgets/` — no Win32 outside `windows*` files. - Own webview windows: `widgets/webview/{mod.rs, windows.rs}`
      (`native_window`, `set_position`, app bar register/unregister, owner, system transitions, WebView2 rasterization
      scale and memory pressure). Handlers use `native_window(..).address()` instead of `webview.hwnd()`. - Windows of
      other apps: `platform::window` gained `set_z_order(ZOrder)`, `place_below`, `close`, `is_resizable`,
      `is_always_on_top`; the wm shadow offset is `outer_rect - inner_rect`. `Process::kill` (taskkill) and
      `platform::shell::{file_app_id, file_display_name, open_native_start_menu}`; the cursor goes through
      `platform::cursor::position`. - `wallpaper_manager`: the `Progman`/`WorkerW` logic is
      `wallpaper_manager/windows.rs` behind `attach_to_desktop(&Window)`. - `weg`: raw win events (native taskbar /
      notification center hiding) and the taskbar pins import are `weg/windows/{raw_events,native_taskbar}.rs`. Accepted
      exception: `platform/hook/windows` still calls `SeelenWeg::process_raw_win_event` (fast path on the raw hook). -
      `toolbar/hook.rs` (commented out dead code) deleted.
- [x] `utils/` — only generic utilities left (the `unsafe` of `lock_free/traced_mutex.rs` is not OS code). `winver.rs` →
      `platform::info` (`has_fixed_runtime`, `is_running_as_appx`, `was_installed_using_msix`; Windows only extras:
      `is_windows_10`, `get_fixed_runtime_path`). `pwsh.rs` and `virtual_desktop.rs` → `windows_api/`,
      `resolve_guid_path` → `WindowsApi::resolve_guid_path`. `integrity`: single instance mutex and UAC/elevation check
      in `integrity/windows.rs`. `icon_extractor` → `resources/icon_extractor/` (it feeds the system icon pack): queue,
      failure cache and image post-processing are agnostic, the per file type / AUMID extraction is
      `icon_extractor/windows.rs` and the HICON/HBITMAP → RGBA conversion is `windows_api/icon.rs` (shared with
      system_tray and window badges). App ids are plain `String`s in the queue (failure cache bumped to
      `icon_failures3.yml`).
- [x] `cli/` — `svc_pipe.rs` launches the service through the new `platform::shell` primitive (`execute`, `open`), which
      replaced every `WindowsApi::execute` outside Windows impls (`exposed.rs`, `utils/integrity`, `widgets/webview.rs`,
      `weg/cli.rs`). `self_pipe.rs`: the `TaskbarButtonChanged` arm (`WindowBadges`, sent by the explorer hook) is
      `#[cfg(windows)]`.
- [x] ~~`hook.rs` (global win event hook)~~ — moved to `platform/hook/` (see Phase 0.5).
- [x] `app.rs`, `main.rs`, `exposed.rs`, `logger.rs`, `telemetry.rs` — isolate Win32 code in `#[cfg(windows)]` functions
      or in a `platform` module. `platform::start()` (background window on Windows + hook + cursor) replaces the 3 calls
      in `app.rs`; the WebView2 fixed runtime env var in `main.rs` is `#[cfg(windows)]`; `select_file_on_explorer` uses
      `platform::shell::reveal`; `get_native_start_menu` moved to the `start` module (`StartMenuSystem::native_layout`);
      the telemetry install id moved to `platform::install_id` (registry on Windows). `logger.rs` needed no changes.

## Phase 3 — Service (`src/service/`) ✔

- [x] Reorganized into `src/service/modules/<module>/{mod.rs, domain.rs, application/{mod.rs, windows.rs}}` (same layout
      as `system_modules`, documented in `service/modules/mod.rs`; `mod.rs` only re-exports the manager, the
      infrastructure is `cli/processing.rs` + `main.rs`):
  - `lifecycle` — `LifecycleSystem { create, init }`, `LifecycleEvent { Suspending, Resumed, EndSession }`: the old
    `power.rs` suspend/resume listener and the shutdown window of `shutdown.rs`. The manager forwards suspend/resume to
    `AppManager` and calls `exit(0)` on end session. The suspend/resume registration is now kept in the native struct
    (released on `Drop`) instead of a thread sleeping in a loop to keep it alive.
  - `app_management` — the crash/suspend aware watchdog is in the manager (`SYSTEM_SUSPENDED` → manager field);
    `AppManagementSystem { launch_app, kill_app_instances }` (explorer de-elevation, session filter) is native.
  - `startup` — `StartupSystem { register_service, set_run_on_logon, unregister_service }` (Task Scheduler).
  - `environment` (typo fixed) — PATH entry add/remove is pure domain logic (`path_with_entry`/`path_without_entry`,
    tested); `EnvironmentSystem { PATH_SEPARATOR, is_path_managed_by_os, system_path, set_system_path }` (registry,
    `REG_EXPAND_SZ`).
  - `taskbar` — the hide worker/request coalescing and pass delays are in the manager; one check of a pass
    (`TaskbarSystem::hide`, app bar state + focus move + hide) and `restore` are native.
  - `hotkeys` — command dispatch (`ShortcutAction`, tested) and the app notifications in the manager; `HotkeysSystem {
    register, stop, start_recording, stop_recording }` wraps `win_hotkeys` (`RecordingEvent { Pressed, Finished }`).
  - `window_positioning` — show/position/foreground native; the batch placement and the `AnimationOrchestrator` are in
    the manager (`libs/positioning` itself is Phase 4).
  - `native_wallpaper` — the worker, saved state, external change detection and restore plan are agnostic (types and
    helpers in domain); `NativeWallpaperSystem` wraps `IDesktopWallpaper` (instance per call, COM init in `init` on the
    worker thread). The persisted `state.json` format is unchanged.
- [x] `cli/processing.rs` is OS-agnostic infrastructure: it only dispatches `SvcAction` to the managers.
- [x] `string_utils.rs` → `windows_api/string_utils.rs` (unused helpers removed); `windows_api/` is only used from
      `windows.rs` files. Process-level primitives (DPI/privileges, single instance mutex, wait for the shell, console,
      local data dir) are in `service/platform/{mod.rs, windows.rs}`. `SERVICE_NAME` (unused) removed,
      `SERVICE_DISPLAY_NAME` is a `&str`.
  - Done: `SvcAction` no longer carries Win32 shaped values (see Phase 4 `SvcAction`).

## Phase 4 — Crates and conditional compilation

- [x] `src/Cargo.toml` (background, service and `slu` CLI): `windows`, `windows-core`, `windows-collections`,
      `windows-future`, `winreg`, `win-screenshot`, `win-hotkeys`, `webview2-com` and `wmi` are
      `[target.'cfg(windows)'.dependencies]`. `widestring`, `battery` and `discord-rich-presence` are cross-platform and
      stay common.
- [x] `libs/positioning` — the animation engine (`lib.rs`: builder, orchestrator, realtime/buffered loops, retargeting)
      is OS-agnostic and talks to the OS through `domain::PositioningSystem` (stateless associated fns: `window_rect`,
      `window_quirks`, `place_window(PlaceOptions)`, `redraw_window`, `boost_current_thread_priority`,
      `create_frame_timer`, `create_overlay`, `cloak`, plus the `FrameTimer`/`ThumbnailOverlay` traits and the `Cloak`
      `Drop` guard as associated types), selected with `#[cfg(windows)] type Platform = windows::WindowsPositioning`.
      The window class checks (Explorer, Chromium, Firefox) became `WindowQuirks { redraw_each_frame,
      pauses_rendering_while_cloaked, breaks_when_cloaked }` built natively (`quirks_for_class`, tested). Win32 lives in
      `windows/{mod, window, overlay, shell_view, timer}.rs` (`SluCloaked` prop and `RECT` conversions included);
      `windows`/`windows-core` are `cfg(windows)` dependencies and `Error::Windows` is `#[cfg(windows)]`. Public API
      unchanged. On `x86_64-unknown-linux-gnu` the only error is the missing `Platform`.
- [x] `libs/slu-ipc` — the transport is the portable `interprocess::local_socket` API (`IpcStream`/`AsyncIpcStream` are
      `local_socket::{Stream, tokio::Stream}`; `common::{connect, connect_async, listen}` with `GenericFilePath` names;
      the server pid comes from `peer_creds()`), so the protocol (framing, timeouts, retries, app/service listeners) and
      the transport are OS-agnostic. On Windows local sockets are the same byte mode duplex named pipes as before, with
      the same `\\.\pipe\seelen-ui[-service]-{session}` paths and security descriptor, so the wire is unchanged.
      `domain::IpcSystem` only covers what local sockets leave to each OS: `current_session_id`,
      `interactive_session_id`, `endpoint(name, session)` (path), `secure_listener` (Windows: the SDDL of
      `windows/security.rs`, no longer public, it had no external users) and `with_io_timeout` (Windows:
      `CancelSynchronousIo` watchdog, as named pipe local sockets have no read/write timeouts). `windows`/`widestring`
      are `cfg(windows)` dependencies and `Error::Windows` is `#[cfg(windows)]`. On `x86_64-unknown-linux-gnu` the only
      error is the missing `Platform`.
  - For a Linux impl: `endpoint` → socket file under `$XDG_RUNTIME_DIR` (not `GenericNamespaced`: the abstract namespace
    has no permissions, any local user could connect), `secure_listener` → `ListenerOptionsExt::mode(0o600)` (optionally
    check `peer_creds().euid()`), `with_io_timeout` → `set_recv_timeout`/`set_send_timeout`, session → the user's uid.
- [x] `libs/slu-ipc` `SvcAction` — the window variants carry domain types: `WindowHandle` (core's existing opaque
      runtime id, no new `WindowId` needed) instead of `isize`, and `WindowState` (moved from `platform::window` to
      `seelen_core::system_state`, re-exported there) instead of the raw `SW_*` command. The wire format is unchanged
      for compatibility between versions: Rust fields `window`/`state` keep the serialized names `hwnd`/`command`, and
      `WindowState` is serialized as the same numbers (`WindowState::code`, frozen, equal to `SW_*`; tested on both
      sides). `SetWindowPosition.flags` was always `0`: it is still sent (previous services require it) but ignored. The
      service's `WindowPositioningSystem` takes `WindowHandle`/`WindowState` (`set_state`, `set_position(rect)`), the
      `WindowState` → `SW_*` mapping lives in its `windows.rs`.
- [x] `libs/utils` — `slu_utils::windows` (`#[cfg(windows)]`, `windows` crate is a `cfg(windows)` dependency) is the
      home of stateless Windows API helpers shared by several binaries (`set_foreground`, `AppBarData`,
      `Com`/`ComThread`, `WindowsString`, `WindowEnumerator`/`MonitorEnumerator`). The duplicated `app_bar`, `com`,
      `iterator` and `string_utils` of the background and the service moved there; errors are `windows::core::Result` or
      generic over the binary error (`Com::run_with_context`, `ComThread<S, E>`). The registered bars tracking stays in
      the background (`widgets/webview/windows.rs`). `cargo check -p slu-utils --target x86_64-unknown-linux-gnu`
      passes.
- [ ] Verify `cargo check` still passes on Windows and, as a goal, that `cargo check --target x86_64-unknown-linux-gnu
      -p seelen-ui` fails only with "trait not implemented" errors (optional empty `application/linux.rs` /
      `unimplemented!()` stub).
  - Done: `cargo clippy --workspace --tests` passes on Windows with no warnings.
  - Pending: it can't be cross-checked from Windows: Tauri's GTK `*-sys` crates (`glib-sys`, `gdk-sys`, ...) need
    `pkg-config` and the Linux system libraries. Run it from WSL / a Linux machine.

## Per-module verification

- `cargo check` (never `--release`).
- Control grep: `rg "windows::|windows_core|winreg|windows_api|HWND|unsafe" src/background/system_modules/*/mod.rs
  src/background/system_modules/*/domain*` → empty.
- `deno task build:rs` in `libs/core` if any exposed type changes (it shouldn't: this is an internal refactor).
- Manual testing by the user in the app (power actions, audio devices, notifications, etc.).
