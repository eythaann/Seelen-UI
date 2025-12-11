# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Seelen UI is a fully customizable desktop environment for Windows built with Rust (backend) and TypeScript/React
(frontend). It provides tiling window management, customizable toolbar, app launcher, media controls, and system
integrations through a Tauri-based architecture.

This project follows a **monorepo architecture** with separate libraries and the main Seelen UI application.

## Build and Development Commands

### Initial Setup

```bash
npm install && npm run dev
```

### Core Development Commands

- `npm run dev` - Start development server with hot reload
- `npm run build:ui` - Build UI components
- `cargo build` - Build Rust backend
- `npm run tauri dev` - Run Tauri in development mode

**CRITICAL BUILD RULE:**

- **NEVER** use `cargo build --release` when testing or type checking
- For type checking, prefer `cargo check` (fastest, no binary generation)
- For testing functionality, use `cargo build` (debug build only)
- Release builds are extremely slow and should only be used for production deployment

### Code Quality (Deno-based)

- `deno lint` - Run Deno linting (replacing ESLint)
- `deno fmt` - Format code with Deno formatter
- `npm run type-check` - Run TypeScript type checking
- `npm test` - Run tests with Jest

### Library Commands (libs/core)

- `cd libs/core && deno task build` - Build core library with TypeScript bindings
- `cd libs/core && deno task build:rs` - Generate Rust-to-TypeScript bindings
- `cd libs/core && deno task build:npm` - Build NPM package from Deno library

### Other Commands

- `npm run translate` - Handle translation tasks
- `npm run tauri:update` - Update Tauri dependencies

## Architecture Overview

### Hybrid Architecture

- **Backend**: Rust with Tauri framework handling system integrations, window management, and native APIs
- **Frontend**: Multiple independent React/Preact applications bundled with esbuild
- **IPC**: Tauri's invoke system for frontend-backend communication
- **Event-Driven**: Async event architecture for real-time state synchronization between backend and frontend widgets

### Monorepo Structure

**Root Level:**

- `libs/` - Shared libraries and core components
  - `core/` - Core Seelen UI library with Rust-generated TypeScript bindings
  - `positioning/` - Positioning utilities
  - `slu-ipc/` - Inter-process communication
  - `widgets-integrity/` - Widget integrity checks
  - `widgets-shared/` - Shared widget components and LazySignal architecture for cross-widget state

**Seelen UI Application:**

- `src/background/` - Rust backend following event-driven architecture
- `src/service/` - System service components
- `src/ui/` - Frontend applications (each subdirectory is an independent app)
  - `settings/` - Main settings application
  - `toolbar/` - System toolbar
  - `weg/` - Taskbar/dock component
  - `launcher/` - App launcher
  - `window_manager/` - Tiling window manager
  - `wallpaper_manager/` - Wallpaper management
  - And more...

### Frontend Applications

Each UI app follows hexagonal architecture with:

- `infra/` - Infrastructure layer (UI components)
- `app/` - Application layer (business logic)
- `domain/` - Domain layer (pure business logic)
- `shared/` - Cross-app utilities and state

### Rust Backend Structure

- **Modules**: System integrations (bluetooth, media, network, power, etc.)
- **Widgets**: UI component handlers (toolbar, launcher, window manager, etc.)
- **State Management**: Centralized application state
- **Windows API**: Native Windows system calls and COM interfaces
- **Virtual Desktops**: Windows virtual desktop integration

## Key Technologies

### Frontend

- **React/Preact** with hooks and functional components
- **Preact Signals** for reactive state management across widgets
- **Redux Toolkit** for complex component-level state management
- **Ant Design** for UI components
- **CSS Modules** for styling
- **i18next** for internationalization (70+ languages supported)
- **Framer Motion** for animations

### Backend

- **Tauri 2.x** as the application framework
- **Windows API** bindings for system integration
- **Tokio** for async runtime
- **Serde** for serialization
- **Parking Lot** for synchronization primitives

### Build System

- **esbuild** for fast frontend bundling
- **Cargo** for Rust compilation
- **TypeScript** with strict type checking
- **Deno** for linting, formatting, and core library management (migrating from Node.js)
- **Rust-to-TypeScript bindings** generated automatically in `libs/core`

## Development Guidelines

### Code Organization

- Follow hexagonal architecture in UI apps - keep `infra/`, `app/`, and `domain/` layers separate
- Use the established import order: infrastructure → app → domain → local files → CSS
- Maintain the modular structure - each UI app is independent
- Keep Rust modules focused on single responsibilities

### Locking Order (Rust)

To prevent deadlocks, always acquire locks in this order:

1. CLI locks
2. DATA locks
3. EVENT locks

### Styling

- Use CSS Modules for component-specific styles
- Follow the established naming conventions (kebab-case for CSS, camelCase for TypeScript)
- Maintain consistency with existing component patterns

### Internationalization

- All user-facing strings must be internationalized
- Add translations to the appropriate YAML files in `i18n/translations/`
- Use the established i18n keys and patterns

### State Management

#### Redux Toolkit (Component State)

- Use Redux Toolkit for complex state in UI apps
- Keep state management in the `store/` directories
- Follow the established patterns for actions, reducers, and selectors

#### LazySignal Architecture (Shared/Global State)

**CRITICAL**: For shared widget state that depends on async events and async getters, use the `LazySignal` pattern from
`libs/widgets-shared/` to avoid race conditions.

**Why LazySignal is Necessary:**

In an event-driven architecture with async initialization and async event handlers, race conditions can occur:

- Initial state fetch may complete after an event has already updated the value
- Multiple async operations may compete to set the initial state
- Event listeners may fire before initialization completes

**How LazySignal Works:**

```typescript
// libs/widgets-shared/LazySignal.ts
class LazySignal<T> extends Signal<T> {
  // Throws error if accessed before initialization
  get value(): T {
    if (!this.initialized) {
      throw new Error("LazySignal was not initialized");
    }
    return this.value;
  }

  // Double-check pattern to prevent race conditions
  async init() {
    if (!this.initialized) {
      const value = await this.initializer();
      // Check again after await - event may have set value during fetch
      if (!this.initialized) {
        this.initialized = true;
        this.value = value;
      }
    }
  }
}
```

**Usage Pattern:**

```typescript
// 1. Create lazy signal with async initializer
const $system_colors = lazySignal(async () => (await UIColors.getAsync()).inner);

// 2. Set up event listeners that can fire anytime
await UIColors.onChange((colors) => ($system_colors.value = colors.inner));

// 3. Initialize - won't overwrite if event already fired
await $system_colors.init();
```

**Key Benefits:**

- **Race Condition Safe**: Double-check pattern ensures events fired during initialization aren't overwritten
- **Fail-Fast**: Throws error if accessed before initialization, catching bugs early
- **Event Handler Utility**: `setByPayload` method simplifies Tauri event integration
- **Type Safe**: Full TypeScript support with proper type inference

**When to Use:**

- ✅ Shared state across multiple widgets/webviews
- ✅ State synchronized with backend via async events
- ✅ Initial state requires async fetch (Tauri invoke, system APIs)
- ✅ State updated by multiple async sources simultaneously
- ❌ Local component state (use React hooks/Redux instead)
- ❌ Simple synchronous state

**Example from Production:**

```typescript
// libs/widgets-shared/signals.ts
const $is_this_webview_focused = lazySignal(() => window.isFocused());
await window.onFocusChanged($is_this_webview_focused.setByPayload);
await $is_this_webview_focused.init();
```

This ensures focus state is always correct regardless of whether:

- The initial fetch completes first, OR
- A focus event fires before initialization completes

## Creating Svelte Widgets

Seelen UI supports creating standalone Svelte widgets that integrate seamlessly with the system. This guide covers the
complete architecture and best practices for creating new widgets.

### Widget Architecture Overview

A complete widget consists of:

1. **Static Widget Definition** (`src/static/widgets/<widget-name>/`)
2. **Svelte UI Application** (`src/ui/svelte/<widget-name>/`)
3. **Theme Styles** (`src/static/themes/default/styles/<widget-name>.scss`)
4. **Internationalization** (i18n translations)
5. **Rust Backend Integration** (optional, for system interactions)

### Directory Structure

```
src/
├── static/
│   └── widgets/
│       └── <widget-name>/          # Widget static definition
│           ├── index.html          # Widget HTML template
│           └── metadata.yml        # Widget metadata
├── ui/
│   └── svelte/
│       └── <widget-name>/          # Svelte application
│           ├── components/         # UI components
│           ├── i18n/              # Translations
│           ├── App.svelte         # Main app component
│           ├── index.ts           # Entry point
│           └── state.svelte.ts    # Global state management
└── static/
    └── themes/
        └── default/
            └── styles/
                └── <widget-name>.scss  # Widget styles
```

### Step-by-Step Widget Creation

#### 1. Create Static Widget Definition

Create `src/static/widgets/<widget-name>/index.html`:

```html
<!DOCTYPE html>
<html>
  <head>
    <meta charset="UTF-8" />
    <title>Widget Name</title>
    <link data-seelen-theme rel="stylesheet" />
    <link data-seelen-styles rel="stylesheet" />
  </head>
  <body>
    <div id="root"></div>
    <script type="module" data-seelen-app></script>
  </body>
</html>
```

Create `src/static/widgets/<widget-name>/metadata.yml`:

```yaml
name: Widget Name
description: Brief description of the widget
author: Your Name
version: 1.0.0
```

#### 2. Create Widget Styles

Create `src/static/themes/default/styles/<widget-name>.scss`:

```scss
@import "shared";

.widget-container {
  @include default-widget-styles;

  // Widget-specific styles
  background: var(--config-background-color);
  color: var(--config-foreground-color);

  .widget-item {
    // Component styles
  }
}
```

**Important CSS Variables:**

- `--system-accent-color` - Accent color
- Use existing mixins from `shared.scss` for consistency

#### 3. Set Up Svelte Application

Create `src/ui/svelte/<widget-name>/index.ts`:

```typescript
import App from "./App.svelte";

const app = new App({
  target: document.getElementById("root")!,
});

export default app;
```

Create `src/ui/svelte/<widget-name>/App.svelte`:

```svelte
<script lang="ts">
  import { globalState } from "./state.svelte";
  // Import components
</script>

<div class="widget-container">
  <!-- Widget UI -->
</div>

<style>
  /* Component-specific styles if needed */
</style>
```

#### 4. Set Up State Management with LazySignal

**CRITICAL**: For widget state that synchronizes with backend events, always use the LazySignal pattern.

Create `src/ui/svelte/<widget-name>/state.svelte.ts`:

```typescript
import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe } from "@seelen-ui/lib";
import type { YourDataType } from "@seelen-ui/lib/types";
import { lazySignal } from "libs/widgets-shared/LazySignal";

// ✅ CORRECT: Use LazySignal for async event-driven state
const $data = lazySignal<YourDataType[]>(async () => {
  return await invoke(SeelenCommand.GetYourData);
});

// Set up event listeners BEFORE initialization
subscribe(SeelenEvent.YourDataChanged, (event) => {
  $data.value = event.payload;
});

// Initialize after setting up listeners
await $data.init();

// Local reactive state
let selectedItem = $state<string | null>(null);
let isLoading = $state(false);

// Export global state
export type State = typeof globalState;
export const globalState = {
  get data() {
    return $data.value;
  },
  get selectedItem() {
    return selectedItem;
  },
  set selectedItem(value: string | null) {
    selectedItem = value;
  },
  get isLoading() {
    return isLoading;
  },
  set isLoading(value: boolean) {
    isLoading = value;
  },
};
```

**When to Use LazySignal:**

✅ **Use LazySignal when:**

- State is fetched asynchronously from backend (Tauri invoke)
- State is updated via backend events (Tauri events)
- Multiple async sources can update the same state
- State is shared across components

❌ **Don't use LazySignal when:**

- State is local to a component (use `$state()`)
- State is synchronous
- State doesn't need backend synchronization

**LazySignal Double-Check Pattern:**

The double-check pattern prevents race conditions:

```typescript
// 1. Create signal with async initializer
const $data = lazySignal(async () => await fetchData());

// 2. Set up event listener (may fire during initialization)
subscribe(SeelenEvent.DataChanged, (e) => {
  $data.value = e.payload; // This sets initialized = true
});

// 3. Initialize (double-check prevents overwriting event data)
await $data.init(); // Only sets value if not already initialized
```

#### 5. Set Up Internationalization

Create `src/ui/svelte/<widget-name>/i18n/index.ts`:

```typescript
import { createI18n } from "libs/ui/svelte/utils/i18n";

const translations = {
  en: {
    key1: "Value 1",
    key2: "Value 2",
  },
  es: {
    key1: "Valor 1",
    key2: "Valor 2",
  },
  // Add all supported languages
};

export const { t, locale } = createI18n(translations);
```

**Translation Best Practices:**

- Always provide translations for all 70+ supported languages
- Use descriptive keys in snake_case
- Keep translations concise and clear
- Test with different language lengths for UI layout

**Using Translations in Components:**

```svelte
<script lang="ts">
  import { t } from "../i18n";
</script>

<button>{$t("button_label")}</button>
<span title={$t("tooltip_text")}>{$t("item_name")}</span>
```

#### 6. Create Widget Components

Create `src/ui/svelte/<widget-name>/components/YourComponent.svelte`:

```svelte
<script lang="ts">
  import type { YourType } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import { t } from "../i18n";
  import { globalState } from "../state.svelte";

  interface Props {
    item: YourType;
    selected: boolean;
    onSelect: () => void;
  }

  let { item, selected, onSelect }: Props = $props();

  let loading = $state(false);
  let error = $state(false);

  // Reset state when selection changes
  $effect(() => {
    if (!selected) {
      loading = false;
      error = false;
    }
  });

  async function handleAction() {
    loading = true;
    error = false;

    try {
      await invoke(SeelenCommand.YourAction, { id: item.id });
    } catch (e) {
      console.error("Action error:", e);
      error = true;
    } finally {
      loading = false;
    }
  }
</script>

<div
  class="component"
  class:selected
  onclick={onSelect}
  role="button"
  tabindex="0"
  onkeydown={() => {}}
>
  <div class="component-info">
    <Icon iconName="YourIcon" />
    <span>{item.name}</span>
  </div>

  {#if selected}
    <div class="component-actions">
      <button onclick={handleAction} disabled={loading}>
        {$t("action_label")}
      </button>
    </div>

    {#if error}
      <div class="component-error">
        {$t("error_message")}
      </div>
    {/if}
  {/if}
</div>

<style>
  .component {
    /* Component styles */
  }

  .component.selected {
    /* Selected state */
  }
</style>
```

#### 7. Rust Backend Integration (Optional)

If your widget needs system integration, create Rust types and commands:

**Define Types** (`libs/core/src/system_state/<feature>/mod.rs`):

```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct YourDataType {
    pub id: String,
    pub name: String,
    pub value: i32,
}

// For enums with tagged representation, use struct variants
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type")] // Use internally tagged representation
#[cfg_attr(feature = "gen-binds", ts(export))]
pub enum YourActionType {
    Simple,
    WithData { data: String }, // ✅ Struct variant
    // ❌ Avoid: WithData(String) - causes serialization errors
}
```

**Add Commands** (`libs/core/src/handlers/commands.rs`):

```rust
slu_commands_declaration! {
    GetYourData = get_your_data() -> Vec<YourDataType>,
    PerformYourAction = perform_your_action(id: String, action: YourActionType),
}
```

**Regenerate TypeScript Bindings:**

```bash
cd libs/core && deno task build:rs
```

### Common Patterns and Best Practices

#### Error Handling

```typescript
async function handleAction() {
  loading = true;
  error = false;

  try {
    const result = await invoke(SeelenCommand.YourAction, { id: item.id });

    if (result) {
      // Handle success
    }
  } catch (e) {
    console.error("Action failed:", e);
    error = true;
  } finally {
    loading = false;
  }
}
```

#### Creating Type-Safe Answer Objects

When building complex objects based on discriminated unions:

```typescript
function createAnswer(): AnswerType {
  // ✅ GOOD: Create base object and modify fields
  const answer: AnswerType = {
    accept: true,
    field1: null,
    field2: null,
    field3: null,
  };

  if (!globalState.action) {
    return answer;
  }

  // Modify only necessary fields
  if (globalState.action.type === "TypeA") {
    answer.field1 = inputValue;
  } else if (globalState.action.type === "TypeB") {
    answer.field2 = otherValue;
  }

  return answer;

  // ❌ BAD: Multiple return statements with full objects
  // Causes code duplication and maintenance issues
}
```

#### Component State Cleanup

Always clean up state when components are deselected:

```svelte
<script lang="ts">
  let loading = $state(false);
  let inputValue = $state("");
  let error = $state(false);

  $effect(() => {
    if (!selected) {
      loading = false;
      inputValue = "";
      error = false;
    }
  });
</script>
```

#### Type Narrowing for Discriminated Unions

```typescript
// TypeScript can narrow types in if blocks
if (action.type === "WithData") {
  // action.data is accessible here
  const data = action.data; // ✅ Type-safe
}
```

```svelte
{#if action.type === "WithData"}
  <!-- action.data may not be narrowed in Svelte templates -->
  <div>{action.data || ""}</div>
{/if}
```

### Checklist for New Widgets

- [ ] Create static widget definition in `src/static/widgets/<widget-name>/`
  - [ ] `index.html` with proper data attributes
  - [ ] `metadata.yml` with widget info
- [ ] Create Svelte application in `src/ui/svelte/<widget-name>/`
  - [ ] `index.ts` entry point
  - [ ] `App.svelte` main component
  - [ ] `state.svelte.ts` with LazySignal for async state
  - [ ] Components in `components/` directory
- [ ] Create widget styles in `src/static/themes/default/styles/<widget-name>.scss`
  - [ ] Use CSS variables for theming
  - [ ] Follow existing style patterns
- [ ] Set up internationalization in `i18n/index.ts`
  - [ ] Add translations for all supported languages
  - [ ] Use descriptive keys
- [ ] If backend integration needed:
  - [ ] Define Rust types in `libs/core/src/system_state/`
  - [ ] Add commands in `libs/core/src/handlers/commands.rs`
  - [ ] Regenerate TypeScript bindings with `deno task build:rs`
  - [ ] Create event handlers in `src/background/modules/`
- [ ] Test the widget
  - [ ] Verify LazySignal prevents race conditions
  - [ ] Test with different languages
  - [ ] Test error states
  - [ ] Verify theme compatibility

### Common Pitfalls to Avoid

1. **Don't use tuple variants in tagged enums**
   ```rust
   // ❌ BAD: Causes serialization errors
   #[serde(tag = "type")]
   pub enum Action {
       WithData(String),
   }

   // ✅ GOOD: Use struct variants
   #[serde(tag = "type")]
   pub enum Action {
       WithData { data: String },
   }
   ```

2. **Don't forget the LazySignal double-check pattern**
   ```typescript
   // ❌ BAD: Race condition possible
   const data = await fetchData();
   subscribe(Event.Changed, (e) => {
     data = e.payload;
   });

   // ✅ GOOD: Double-check pattern
   const $data = lazySignal(async () => await fetchData());
   subscribe(Event.Changed, (e) => {
     $data.value = e.payload;
   });
   await $data.init();
   ```

3. **Don't create multiple return objects**
   - Creates a base object and modify it (see "Creating Type-Safe Answer Objects")

4. **Don't forget to clean up component state**
   - Use `$effect()` to reset state when components are deselected

### Windows Integration

- Use the existing Windows API abstractions in `src/background/windows_api/`
- Follow COM object management patterns for system interactions
- Respect Windows-specific behaviors and constraints

## Testing

- Run `npm test` to execute Jest tests
- Ensure new features include appropriate test coverage
- Test UI changes across different Windows versions when possible

## Dependencies

- **Core Library**: Uses `@seelen-ui/lib` from `libs/core` (monorepo library with Rust-generated TypeScript bindings)
- **Tauri Plugins**: Extensive use of official Tauri plugins for system integration
- **Windows Runtime**: Requires WebView2 and Microsoft Edge for proper functionality
- **Deno Runtime**: Progressive migration from Node.js to Deno for tooling and core library

## Building and Deployment

- Production builds require both frontend and backend compilation
- Uses NSIS for Windows installer creation
- Supports multiple installation methods (Microsoft Store, Winget, direct download)
- Code signing configured for release builds
