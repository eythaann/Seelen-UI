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
