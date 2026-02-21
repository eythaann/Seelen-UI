# AGENTS.md

Guidance for AI agents working in this repository.

Seelen UI is a customizable Windows desktop environment built with:

- Rust + Tauri (backend)
- TypeScript + React/Preact (frontend)
- A monorepo layout with shared libs under `libs/`

## Read First (Non-Negotiable)

Build speed / safety:

- DO NOT use `cargo build --release` for testing, type-checking, or local iteration.
- Prefer `cargo check` for fast Rust validation.
- Use `cargo build` (debug) only when you need a binary.

Translations:

- DO NOT run `npm run translate` during active development.
- Add translations manually while iterating; run the translate command only right before a final commit.

Rust locking order (avoid deadlocks):

1. CLI locks
2. DATA locks
3. EVENT locks

Backend architecture rules:

- System modules in `src/background/modules/` MUST follow the modern pattern (lazy init + lazy tauri registration).
- Business logic must NOT call `emit_to_webviews` directly.

WinRT / COM safety:

- For WinRT objects with event subscriptions, use wrapper structs with `Drop` for automatic unregistration.
- Windows-rs clones `TypedEventHandler` internally: store tokens, not handlers.

## Common Commands

Initial setup:

```bash
npm install && npm run dev
```

Dev / build:

- `npm run dev` - Frontend dev workflow
- `npm run build:ui` - Build UI bundles
- `npm run tauri dev` - Run Tauri in dev mode
- `cargo check` - Fast Rust type check
- `cargo build` - Debug build

Quality (Deno-based):

- `deno lint`
- `deno fmt`
- `npm run type-check`
- `npm test`

Core library (`libs/core`):

- `deno task build`
- `deno task build:rs` - Regenerate Rust -> TypeScript bindings
- `deno task build:npm`

## Repo Map (Where Things Live)

Shared libraries:

- `libs/core/` - Core library + Rust-generated TypeScript bindings
- `libs/widgets-shared/` - Cross-widget state utilities (includes LazySignal)
- `libs/slu-ipc/`, `libs/positioning/`, `libs/widgets-integrity/`

Main app:

- `src/background/` - Rust backend (modules, native integrations)
- `src/service/` - System service components
- `src/ui/` - Frontend apps (each subdirectory is an independent app)
  - examples: `src/ui/settings/`, `src/ui/toolbar/`, `src/ui/launcher/`, `src/ui/window_manager/`

## Frontend Conventions

App architecture:

- UI apps use a hexagonal-ish layering: `infra/`, `app/`, `domain/`, `shared/`.
- Keep boundaries clean: `domain/` is pure logic; `infra/` is UI + integration.

Styling:

- CSS Modules are the default.
- Naming: kebab-case for CSS, camelCase for TS.

Internationalization:

- All user-visible strings must be i18n.
- Translation files live under `i18n/translations/` (YAML).

## Backend: System Modules (Modern Pattern)

All modules in `src/background/modules/` follow this pattern:

- `application.rs` owns the singleton manager and emits internal events.
- `infrastructure.rs` (or `handlers.rs`) owns Tauri commands and bridges internal events -> webviews.
- Tauri event registration happens lazily on first command access (via `Once`).

Suggested layout:

```
src/background/modules/<module>/
  mod.rs
  application.rs
  infrastructure.rs  # or handlers.rs
  domain.rs          # optional
```

Minimal pattern (infrastructure side):

```rust
use std::sync::Once;
use seelen_core::handlers::SeelenEvent;
use crate::{app::emit_to_webviews, error::Result};
use super::{YourEvent, YourManager};

fn get_manager() -> &'static YourManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        YourManager::subscribe(|_event: YourEvent| {
            // Keep this small and side-effect focused.
            if let Ok(data) = get_your_data() {
                emit_to_webviews(SeelenEvent::YourDataChanged, data);
            }
        });
    });
    YourManager::instance()
}

#[tauri::command(async)]
pub fn get_your_data() -> Result<Vec<YourType>> {
    let manager = get_manager();
    Ok(manager.get_data())
}
```

Minimal pattern (application side):

```rust
use std::sync::LazyLock;

pub struct YourManager {
    // fields
}

#[derive(Debug, Clone)]
pub enum YourEvent {
    DataChanged,
}

event_manager!(YourManager, YourEvent);

impl YourManager {
    fn new() -> Self {
        Self { /* init */ }
    }

    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<YourManager> = LazyLock::new(|| {
            let mut m = YourManager::new();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn init(&mut self) -> Result<()> {
        self.setup_listeners()?;
        Ok(())
    }

    fn setup_listeners(&mut self) -> Result<()> {
        // Listen to OS signals; emit internal YourEvent::* (not webview events)
        Ok(())
    }

    pub fn get_data(&self) -> Vec<YourType> {
        // return data
        vec![]
    }
}
```

When adding a new backend feature exposed to the UI, update `libs/core`:

1. `libs/core/src/handlers/commands.rs`

```rust
slu_commands_declaration! {
    GetYourData = get_your_data() -> Vec<YourType>,
}
```

2. `libs/core/src/handlers/events.rs`

```rust
slu_events_declaration! {
    YourDataChanged(Vec<YourType>) as "your-module::data-changed",
}
```

3. Regenerate bindings: `cd libs/core && deno task build:rs`

## WinRT Wrapper Pattern (Automatic Cleanup)

Use wrappers for WinRT objects that register events.

Rules:

- Store event tokens (WinRT tokens are often `i64`).
- Do NOT store `TypedEventHandler` values in struct fields.
- Implement `Drop` to unregister events.

Example:

```rust
pub struct WinRtWrapper {
    pub object: SomeWinRtObject,
    token: i64,
}

impl WinRtWrapper {
    pub fn create(object: SomeWinRtObject) -> Result<Self> {
        let token = object.SomeEvent(&TypedEventHandler::new(Self::on_event))?;
        Ok(Self { object, token })
    }

    fn on_event(
        _sender: &Option<SomeWinRtObject>,
        _args: &Option<SomeArgs>,
    ) -> windows_core::Result<()> {
        Ok(())
    }
}

impl Drop for WinRtWrapper {
    fn drop(&mut self) {
        self.object.RemoveSomeEvent(self.token).log_error();
    }
}
```

## Shared State: LazySignal (Cross-Widget)

Use `LazySignal` (in `libs/widgets-shared/`) when state is:

- fetched asynchronously (invoke/system APIs)
- updated by async events
- shared across widgets/webviews

Critical usage pattern:

1. Create lazy signal with async initializer.
2. Register event listeners first (they may fire immediately).
3. Call `.init()` last; it must not overwrite a value set by an event.

Example:

```ts
import { lazySignal } from "libs/widgets-shared/LazySignal";
import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";

const $data = lazySignal(async () => {
  return await invoke(SeelenCommand.GetYourData);
});

subscribe(SeelenEvent.YourDataChanged, (event) => {
  $data.value = event.payload;
});

await $data.init();
```

## Creating Svelte Widgets (High-Level)

Seelen UI supports standalone Svelte widgets. Prefer following existing widget patterns; do not invent new build
plumbing.

Typical pieces:

1. Static widget definition: `src/static/widgets/<widget-name>/`
2. Svelte app: `src/ui/svelte/<widget-name>/`
3. Theme styles: `src/static/themes/default/styles/<widget-name>.scss`
4. i18n: translations (and keep the translation workflow rule)
5. Optional Rust backend integration (use the modern module pattern)

Widget checklist:

- Static metadata and HTML exist under `src/static/widgets/<widget-name>/`
- Svelte entry mounts into `#root` and calls `Widget.getCurrent().init(...)`
- Shared, event-driven state uses LazySignal
- Styling uses existing CSS variables; avoid global class conflicts

Shared styling for widgets:

- Use `data-skin` attributes for common control styling (buttons, inputs) to avoid class collisions.

## Rust Types: Tagged Enums (Serde)

Avoid tuple variants for internally tagged enums.

Bad:

```rust
#[serde(tag = "type")]
pub enum Action {
    WithData(String),
}
```

Good:

```rust
#[serde(tag = "type")]
pub enum Action {
    WithData { data: String },
}
```

## Testing Expectations

- Prefer quick feedback loops (`cargo check`, `npm run type-check`, `deno lint`).
- Keep changes scoped; add tests when behavior changes.
