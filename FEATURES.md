# Seelen UI — Feature Reference

Seelen UI is a fully customizable Windows desktop environment built on Rust + Tauri (backend) and TypeScript +
React/Preact/Svelte (frontend). Every major component is a **widget** — a sandboxed WebView with its own settings,
shortcuts, and theme overrides.

---

## Table of Contents

**Core Resources**

1. [Global Settings](#global-settings)
2. [Themes](#themes)
3. [Icon Packs](#icon-packs)
4. [Sound Packs](#sound-packs)
5. [Wallpaper Manager](#wallpaper-manager)

**Desktop Environment** 6. [Virtual Desktops](#virtual-desktops) 7.
[SeelenWeg — Dock / Taskbar](#seelenweg--dock--taskbar) 8. [Fancy Toolbar](#fancy-toolbar) 9.
[Window Manager (Tiling WM)](#window-manager-tiling-wm) 10. [Apps Menu (Start Menu)](#apps-menu-start-menu)

**Overlay Widgets** 11. [Task Switcher](#task-switcher) 12. [Workspaces Viewer](#workspaces-viewer) 13.
[Popup Widget](#popup-widget)

**Status & Notification Widgets** 14. [Quick Settings](#quick-settings) 15. [Notifications](#notifications) 16.
[Clipboard History](#clipboard-history) 17. [Flyouts](#flyouts)

**Popup Widgets** 18. [Media Popup](#media-popup) 19. [Network Popup](#network-popup) 20.
[Bluetooth Popup](#bluetooth-popup) 21. [Calendar Popup](#calendar-popup) 22. [Power Menu](#power-menu) 23.
[User Menu](#user-menu)

**Toolbar System Widgets** 24. [System Tray](#system-tray) 25. [Keyboard Selector](#keyboard-selector)

**Shared UI Components** 26. [Context Menu](#context-menu) 27. [Tooltip](#tooltip)

**Configuration & System** 28. [Settings Window](#settings-window) 29. [Plugins System](#plugins-system) 30.
[Per-Monitor Configuration](#per-monitor-configuration) 31.
[App Rules (Per-App Configuration)](#app-rules-per-app-configuration) 32. [Shortcuts](#shortcuts) 33.
[Performance Mode](#performance-mode) 34. [Backup & Sync](#backup--sync) 35. [Developer & Extras](#developer--extras)

---

## Global Settings

Settings that apply to the entire application (stored in the `Settings` struct):

| Setting                 | Description                                                                     |
| ----------------------- | ------------------------------------------------------------------------------- |
| `language`              | UI language; `null` uses the system locale                                      |
| `dateFormat`            | MomentJS date format string (e.g. `"YYYY-MM-DD"`)                               |
| `startOfWeek`           | First day of week for calendars: `Monday`, `Sunday`, or `Saturday`              |
| `activeThemes`          | Ordered list of active theme IDs (stacked)                                      |
| `activeIconPacks`       | Ordered list of active icon pack IDs                                            |
| `devTools`              | Show/hide the Developer Tools tab in Settings                                   |
| `drpc`                  | Discord Rich Presence integration on/off                                        |
| `streamingMode`         | Replaces sensitive info (email, names) with placeholders in recordings          |
| `hardwareAcceleration`  | Enable/disable GPU acceleration for WebViews                                    |
| `unstableOptimizations` | Chromium `--process-per-site` flag (may reduce RAM, risk of crashes)            |
| `pollingInterval`       | Seconds between system resource polls (CPU, memory, network)                    |
| `suspendOnGameMode`     | Suspend all WebViews when Windows Game Mode is active                           |
| `backupSyncEnabled`     | Automatic cloud backup sync on/off                                              |
| `performanceMode`       | Performance profile per power state (see [Performance Mode](#performance-mode)) |
| `shortcuts`             | Global shortcut enable/disable + key overrides                                  |
| `updater.channel`       | Update channel: `Stable`, `Beta`, or `Nightly`                                  |
| `byWidget`              | Per-widget settings overrides (merged with widget defaults)                     |
| `byTheme`               | Per-theme CSS variable overrides                                                |
| `byWallpaper`           | Per-wallpaper display settings                                                  |
| `wallpaperCollections`  | Named wallpaper collections                                                     |
| `monitorsV3`            | Per-monitor configuration dictionary                                            |

---

## Themes

Themes apply CSS to individual widgets and shared component styles (buttons, inputs, scrollbars, etc.).

### Theme Structure

- **ID** — unique resource ID (e.g. `@username/my-theme`)
- **Metadata** — display name, description, tags, preview images
- **`styles`** — a map of `widgetId → CSS string`; only the widgets listed receive styles
- **`sharedStyles`** — CSS injected into all widgets (suitable for component library overrides)
- **`settings`** — a typed settings declaration (`ThemeSettingsDefinition`) that generates a UI panel in Settings
  allowing end-users to customize CSS variables exposed by the theme

### Theme Variable Types

Themes can declare the following variable types for user customization:

| Syntax                | What it controls                        |
| --------------------- | --------------------------------------- |
| `<string>`            | Arbitrary text value                    |
| `<color>`             | Color picker                            |
| `<length-percentage>` | Numeric value with unit (px, rem, %, …) |
| `<number>`            | Plain number                            |
| `<url>`               | URL string                              |
| `<family-name>`       | Font family picker                      |

### System CSS Variables (always available)

Themes and custom CSS can reference these variables injected by the engine:

| Variable                         | Source                                   |
| -------------------------------- | ---------------------------------------- |
| `--system-accent-color`          | Windows accent color                     |
| `--system-accent-light-color`    | Accent light variant                     |
| `--system-accent-lighter-color`  | Accent lighter variant                   |
| `--system-accent-lightest-color` | Accent lightest variant                  |
| `--system-accent-dark-color`     | Accent dark variant                      |
| `--system-accent-darker-color`   | Accent darker variant                    |
| `--system-accent-darkest-color`  | Accent darkest variant                   |
| `--color-gray-*`                 | Gray scale palette (50–900)              |
| `--slu-std-ui-color`             | Adaptive accent color (light/dark aware) |

### Bundled Themes

| Theme   | ID                 | Targets          |
| ------- | ------------------ | ---------------- |
| Default | `@default/theme`   | All core widgets |
| Bubbles | `@default/bubbles` | Fancy Toolbar    |

### Theme Per-Widget Customization

Each theme can independently style any of the following widgets:

- `@seelen/fancy-toolbar`
- `@seelen/weg`
- `@seelen/window-manager`
- `@seelen/wallpaper-manager`
- `@seelen/apps-menu`
- `@seelen/quick-settings`
- `@seelen/bluetooth-popup`
- `@seelen/network-popup`
- `@seelen/media-popup`
- `@seelen/calendar-popup`
- `@seelen/notifications`
- `@seelen/power-menu`
- `@seelen/task-switcher`
- `@seelen/system-tray`
- `@seelen/workspaces-viewer`
- `@seelen/user-menu`
- `@seelen/keyboard-selector`
- `@seelen/context-menu`
- `@seelen/flyouts`
- `@seelen/tooltip`
- Any third-party widget by its ID

### Multiple Themes (Stacking)

Multiple themes can be active simultaneously. Styles are applied in order — later themes override earlier ones. This
enables a "base theme + accent theme" workflow.

### Theme Settings UI

The Settings window renders a full configuration panel for each active theme's declared variables, allowing users to
override colors, sizes, fonts, etc., without editing CSS.

---

## Icon Packs

Icon packs replace the default application icons shown in the dock, taskbar, and Start Menu.

- Multiple icon packs can be active simultaneously (first match wins)
- Entries match by application path, UMID, or process name
- A `missing` icon fallback is supported for unmatched apps
- Remote entries are downloaded and cached locally
- The **Icon Pack Editor** in Settings allows creating/editing custom icon packs
- Icons can be exported per-app as custom overrides

---

## Sound Packs

Sound Packs are a resource kind (`ResourceKind.SoundPack`) defined in the core library and listed as a section in the
Settings Resources navigation. The feature is currently **not implemented** — the Settings component returns `null` and
no audio playback infrastructure exists.

This is a placeholder for a future feature that would allow replacing system sound effects with custom audio packs.

---

## Wallpaper Manager

ID: `@seelen/wallpaper-manager`

Full-featured animated desktop background engine that replaces the Windows wallpaper system.

### Wallpaper Types

| Type          | Description                                      |
| ------------- | ------------------------------------------------ |
| `Image`       | Static image file (JPG, PNG, WebP, etc.)         |
| `Video`       | Video file played as wallpaper (MP4, WebM, etc.) |
| `Layered`     | Custom HTML/CSS/JS layer rendered as wallpaper   |
| `MediaPlayer` | Synced with system media playback                |
| `Unsupported` | Unrecognized format                              |

### Wallpaper Settings (global)

| Setting                 | Description                                                       |
| ----------------------- | ----------------------------------------------------------------- |
| `enabled`               | Enable/disable the wallpaper engine                               |
| `interval`              | Rotation interval in seconds (min 60 s)                           |
| `randomize`             | Randomize wallpaper order                                         |
| `defaultCollection`     | Default wallpaper collection ID                                   |
| `multimonitorBehaviour` | `PerMonitor` (independent) or `Extend` (panoramic)                |
| `useAccentColor`        | Extract and apply Windows accent color from the current wallpaper |

### Per-Wallpaper Instance Settings

Each wallpaper can be individually configured:

| Setting               | Options / Range                                                         |
| --------------------- | ----------------------------------------------------------------------- |
| `playbackSpeed`       | Video playback speed multiplier                                         |
| `flipVertical`        | Flip image/video vertically                                             |
| `flipHorizontal`      | Flip image/video horizontally                                           |
| `blur`                | Blur intensity (0–∞)                                                    |
| `objectFit`           | `Fill`, `Contain`, `Cover`, `None`, `ScaleDown`                         |
| `objectPosition`      | CSS-style position (e.g. `center center`)                               |
| `saturation`          | Color saturation (0–2, default 1)                                       |
| `contrast`            | Contrast level (0–2, default 1)                                         |
| `withOverlay`         | Enable color overlay on top of the wallpaper                            |
| `overlayColor`        | Overlay color (any CSS color)                                           |
| `overlayMixBlendMode` | CSS mix-blend-mode for the overlay                                      |
| `muted`               | Mute video wallpaper audio                                              |
| `css`                 | Custom CSS injected only for this wallpaper (Layered/MediaPlayer types) |

### Wallpaper Collections

Wallpapers can be organized into named collections. Each monitor and each virtual workspace can independently select
which collection to display.

### Shortcuts

| Action             | Default Keys        |
| ------------------ | ------------------- |
| Next wallpaper     | `Ctrl + Win + Up`   |
| Previous wallpaper | `Ctrl + Win + Down` |

---

## Virtual Desktops

Seelen UI implements a per-monitor virtual desktop system that replaces and extends the Windows 10/11 virtual desktop
experience.

### Architecture

- Workspaces are tracked **per monitor** — each monitor has its own independent list of workspaces
- A `VirtualDesktops` object stores `monitors: { [MonitorId]: VirtualDesktopMonitor }` and a global `pinned: number[]`
  list of pinned HWNDs
- Pinned windows appear on every workspace on every monitor and are never moved when switching

### Workspace Properties

| Property    | Description                                                                    |
| ----------- | ------------------------------------------------------------------------------ |
| `id`        | Unique workspace identifier (UUID)                                             |
| `name`      | Optional human-readable label shown in the Workspaces Viewer and Fancy Toolbar |
| `icon`      | Optional React Icon name displayed in toolbar workspace indicators             |
| `wallpaper` | Optional wallpaper ID — overrides the monitor collection for this workspace    |
| `windows`   | List of HWNDs currently assigned to this workspace                             |

### Workspace Management

- Create, rename, and destroy workspaces via keyboard shortcuts or the Workspaces Viewer
- Move a focused window to any workspace (with or without following it)
- Send a window to a workspace and immediately switch to it
- Workspaces are listed in the Fancy Toolbar items (`Workspaces (Dotted)`, `(Named)`, `(Numbered)`)
- The `vdMain` system shortcut group controls create/destroy/rename
- `vdSwitch`, `vdMove`, `vdSend` groups control navigation and window movement

### Wallpaper Integration

Each workspace can display a different wallpaper collection, enabling per-workspace visual identities. The priority is:

1. Workspace-level wallpaper override (most specific)
2. Monitor-level wallpaper collection
3. Global default wallpaper collection

---

## SeelenWeg — Dock / Taskbar

ID: `@seelen/weg`\
Instances: one per monitor (ReplicaByMonitor)

A fully featured dock/taskbar replacement supporting pinned apps, running windows, and special items.

### Dock Settings

| Setting                   | Options / Range                                                                  |
| ------------------------- | -------------------------------------------------------------------------------- |
| `enabled`                 | Enable/disable                                                                   |
| `mode`                    | `Full-Width` (taskbar-style) or `Centered` (dock-style)                          |
| `position`                | `Top`, `Bottom`, `Left`, `Right`                                                 |
| `hideMode`                | `Never`, `Always` (auto-hide), `OnOverlap` (hide when windows overlap)           |
| `delayToShow`             | Milliseconds before showing on hover                                             |
| `delayToHide`             | Milliseconds before hiding on mouse leave                                        |
| `margin`                  | Outer margin in px                                                               |
| `padding`                 | Inner padding in px                                                              |
| `size`                    | Icon size in px                                                                  |
| `zoomSize`                | Zoomed icon size in px (hover animation)                                         |
| `spaceBetweenItems`       | Gap between items in px                                                          |
| `showWindowTitle`         | Show window title next to open app icons                                         |
| `showInstanceCounter`     | Show badge with count of open windows per app                                    |
| `visibleSeparators`       | Show visual separators between sections                                          |
| `splitWindows`            | Show each window as a separate item instead of grouping by app                   |
| `temporalItemsVisibility` | `All` (show running apps from all monitors) or `OnMonitor` (only this monitor)   |
| `pinnedItemsVisibility`   | `Always` (show pinned items on every monitor) or `WhenPrimary` (only on primary) |
| `middleClickAction`       | `CloseApp` or `OpenNewInstance`                                                  |
| `showEndTask`             | Add "End Task" option in context menu (requires Dev Mode)                        |

### Dock Item Types

| Type          | Description                             |
| ------------- | --------------------------------------- |
| `AppOrFile`   | Application or file shortcut (pinnable) |
| `Separator`   | Visual divider between sections         |
| `Media`       | Inline media player controls            |
| `StartMenu`   | Start Menu button                       |
| `ShowDesktop` | Show/hide all windows button            |
| `TrashBin`    | Recycle Bin with item count badge       |

### Pinned Item Properties

- `displayName` — shown in tooltips
- `path` — file or executable path
- `umid` — Application User Model ID (for packaged apps)
- `pinned` — persists when all windows are closed
- `preventPinning` — disables the pin option
- `relaunch` — custom relaunch command

### Window Preview

Hovering over a dock item with open windows shows a thumbnail preview panel with the window title and a live preview of
the window content.

### Context Menu per Item

Right-clicking a dock item opens a context menu with options:

- Open / Focus window
- Pin / Unpin
- Close window
- Open new instance
- End task (if Dev Mode enabled)
- Open file location

### Shortcuts

| Action           | Default Keys          |
| ---------------- | --------------------- |
| Launch item 1–10 | `Win + 1` … `Win + 0` |

---

## Fancy Toolbar

ID: `@seelen/fancy-toolbar`\
Instances: one per monitor (ReplicaByMonitor)

A fully scriptable status bar where each item is powered by JavaScript and can display any data available in the system
scopes.

### Toolbar Settings

| Setting       | Options / Range                           |
| ------------- | ----------------------------------------- |
| `enabled`     | Enable/disable                            |
| `position`    | `Top`, `Bottom`, `Left`, `Right`          |
| `itemSize`    | Height/width of each item in px           |
| `padding`     | Inner padding in px                       |
| `margin`      | Outer margin in px                        |
| `hideMode`    | `Never`, `Always`, `OnOverlap`            |
| `delayToShow` | Milliseconds before showing on hover      |
| `delayToHide` | Milliseconds before hiding on mouse leave |

### Toolbar Item Definition

Each toolbar item is a data record with:

| Field        | Description                                                     |
| ------------ | --------------------------------------------------------------- |
| `id`         | Unique UUID                                                     |
| `scopes`     | List of data scopes to inject into the JS execution environment |
| `template`   | JS function returning the rendered HTML content                 |
| `tooltip`    | JS function returning tooltip HTML                              |
| `badge`      | JS function returning badge/notification content                |
| `onClick`    | JS function executed on click                                   |
| `style`      | React-style CSS object for item container                       |
| `remoteData` | Remote URLs fetched and injected into scope                     |

### Available JS Data Scopes

| Scope               | Provides                                  |
| ------------------- | ----------------------------------------- |
| `Date`              | Current date/time                         |
| `Notifications`     | Notification list and count               |
| `Media`             | Active media players and output devices   |
| `Network`           | Network adapters and connectivity         |
| `Keyboard`          | Active keyboard layout                    |
| `User`              | Current user profile info                 |
| `Bluetooth`         | Bluetooth adapter and paired devices      |
| `Power`             | Battery level, charging state, power mode |
| `FocusedApp`        | Currently focused application             |
| `Workspaces`        | Virtual desktops and workspace info       |
| `Disk`              | Disk usage per drive                      |
| `NetworkStatistics` | Real-time upload/download speeds          |
| `Memory`            | RAM usage                                 |
| `Cpu`               | Per-core CPU usage                        |

### Toolbar Sections

The toolbar has three zones: **Left**, **Center**, **Right**. Items are assigned to zones and can be reordered via
drag-and-drop in the Settings editor.

### Built-in Toolbar Plugins

These are pre-built toolbar item templates included with Seelen UI:

| Plugin                | Displays                                          |
| --------------------- | ------------------------------------------------- |
| CPU Usage             | Per-core average CPU utilization (%)              |
| Memory Usage          | RAM used / total                                  |
| Disk Usage            | Disk read/write activity                          |
| Network Usage         | Real-time network upload + download               |
| Power (Battery)       | Battery percentage + charging status with tooltip |
| Focused App           | Icon of currently focused application             |
| Focused App Title     | Title of currently focused window                 |
| Workspaces (Dotted)   | Virtual desktop indicators as dots                |
| Workspaces (Named)    | Virtual desktop names as buttons                  |
| Workspaces (Numbered) | Virtual desktop numbers as buttons                |

### Widget Plugins (Toolbar Integrations)

The following widgets install a toolbar plugin that adds a clickable toolbar item to trigger the popup:

| Widget            | Default Trigger                      |
| ----------------- | ------------------------------------ |
| Calendar Popup    | Click opens date/calendar popup      |
| Media Popup       | Click opens volume/media popup       |
| Network Popup     | Click opens Wi-Fi popup              |
| Bluetooth Popup   | Click opens Bluetooth popup          |
| Quick Settings    | Click opens quick settings           |
| Notifications     | Click opens notification center      |
| Power Menu        | Click opens power/session menu       |
| System Tray       | System tray icon area                |
| User Menu         | Click opens user profile popup       |
| Keyboard Selector | Click opens keyboard layout switcher |

---

## Window Manager (Tiling WM)

ID: `@seelen/window-manager`\
Instances: one per monitor (ReplicaByMonitor)

A full tiling window manager that automatically arranges windows into configurable layouts without overlapping.

### Window Manager Settings

| Setting                   | Options / Range                                               |
| ------------------------- | ------------------------------------------------------------- |
| `enabled`                 | Enable/disable                                                |
| `defaultLayout`           | Plugin ID of the default layout                               |
| `autoStackingByCategory`  | Automatically group windows of the same category into stacks  |
| `border.enabled`          | Show a colored border around managed windows                  |
| `border.width`            | Border width in px                                            |
| `border.offset`           | Border offset (negative = inside window)                      |
| `resizeDelta`             | Resize step size in % (1–40)                                  |
| `workspaceGap`            | Gap between tiled containers in px                            |
| `workspacePadding`        | Inner padding of the workspace in px                          |
| `workspaceMargin`         | Outer margin (`top/right/bottom/left`) in px                  |
| `floating.width`          | Default width for floating windows in px                      |
| `floating.height`         | Default height for floating windows in px                     |
| `animations.enabled`      | Enable/disable window movement animations                     |
| `animations.durationMs`   | Animation duration in ms                                      |
| `animations.easeFunction` | CSS easing function (e.g. `ease-in-out`)                      |
| `dragBehavior`            | `Sort` (insert at drop target) or `Swap` (exchange positions) |

### Built-in Layout Plugins

| Layout | ID                  | Description                                                         |
| ------ | ------------------- | ------------------------------------------------------------------- |
| BSPWM  | `@default/wm-bspwm` | Binary space partition — recursive horizontal/vertical split        |
| Tall   | `@default/wm-tall`  | One wide primary pane + vertical stack (good for portrait monitors) |
| Wide   | `@default/wm-wide`  | One tall primary pane + horizontal stack                            |
| Grid   | `@default/wm-grid`  | Dynamic grid that adapts as windows are added                       |

### Layout Node Types

Layouts are trees of `WmNode` with these types:

| Type         | Description                                 |
| ------------ | ------------------------------------------- |
| `Leaf`       | Single window slot                          |
| `Stack`      | Multiple windows tabbed/stacked in one slot |
| `Vertical`   | Children split vertically (top/bottom)      |
| `Horizontal` | Children split horizontally (left/right)    |

Each node supports:

- `growFactor` — relative size weight
- `condition` — math expression controlling visibility (e.g. `n >= 3`)
- `priority` — traversal order
- `maxStackSize` — cap on stacked windows (Stack nodes only)
- `lifetime` — `Permanent`, `Ephemeral`, etc.

### Window Modes (per window)

| Mode            | Behavior                                    |
| --------------- | ------------------------------------------- |
| Tiled (default) | Managed by the layout                       |
| Float           | Free-floating window, not managed by layout |
| Monocle         | Maximized within workspace bounds           |

### Keyboard Shortcuts

| Action                          | Default Keys      |
| ------------------------------- | ----------------- |
| Toggle WM on/off                | `Win + P`         |
| Toggle float for focused window | `Win + F`         |
| Toggle monocle                  | `Win + M`         |
| Cycle stack (next)              | `Win + Q`         |
| Cycle stack (previous)          | `Win + Shift + Q` |
| Reserve top                     | `Win + Shift + I` |
| Reserve bottom                  | `Win + Shift + K` |
| Reserve left                    | `Win + Shift + J` |
| Reserve right                   | `Win + Shift + L` |
| Reserve float                   | `Win + Shift + U` |
| Reserve stack                   | `Win + Shift + O` |
| Focus up                        | `Alt + I`         |
| Focus down                      | `Alt + K`         |
| Focus left                      | `Alt + J`         |
| Focus right                     | `Alt + L`         |
| Increase width                  | `Win + Alt + =`   |
| Decrease width                  | `Win + Alt + -`   |
| Increase height                 | `Win + Ctrl + =`  |
| Decrease height                 | `Win + Ctrl + -`  |
| Reset all sizes                 | `Win + Alt + 0`   |
| Move window up                  | `Shift + Alt + I` |
| Move window down                | `Shift + Alt + K` |
| Move window left                | `Shift + Alt + J` |
| Move window right               | `Shift + Alt + L` |

---

## Apps Menu (Start Menu)

ID: `@seelen/apps-menu`\
Trigger: `Win` key (system shortcut, intercepted from Windows)

A full replacement for the Windows Start Menu.

### Views

- **Pinned View** — shows apps the user has manually pinned, organized in a grid
- **All Apps View** — alphabetical list of all installed applications
- Toggled with the "All" / "Back" button

### Search

The search bar filters apps in real time. Prefixes change search scope:

| Prefix   | Scope                                  |
| -------- | -------------------------------------- |
| _(none)_ | All results                            |
| `apps:`  | Only installed applications            |
| `files:` | File system search                     |
| `web:`   | Opens Google Search in default browser |

Pressing `Enter` launches the first/selected result. Arrow keys navigate the grid.

### Pinned Items

- Apps can be pinned/unpinned via right-click
- Pinned items are drag-and-drop reorderable
- Apps can be grouped into **folders** — drag one app onto another to create a folder
- Folders can be renamed, apps can be added/removed
- Two folders can be merged; a folder can be disbanded back into individual items
- Pinned state is persistent (survives restarts)
- Initial pinned list is imported from the native Windows Start Menu on first run

### Footer Actions

| Button               | Action                           |
| -------------------- | -------------------------------- |
| User avatar/name     | Opens User Menu popup            |
| Settings gear        | Opens Seelen UI Settings         |
| Power icon           | Opens Power Menu                 |
| Expand/Collapse icon | Toggles fullscreen / normal mode |

### Acrylic Effect

Supports Acrylic (Mica-like) blur effect, configurable via widget settings:

| Setting   | Description                                |
| --------- | ------------------------------------------ |
| `acrylic` | Enable Windows Acrylic transparency effect |

---

## Task Switcher

ID: `@seelen/task-switcher`\
Hidden overlay triggered by `Alt + Tab`.

Visual replacement for the Windows Alt+Tab window switcher.

### Features

- Thumbnail preview of each open window
- Keyboard navigation (arrow keys, Tab to cycle)
- Auto-confirm mode: pressing `Alt + Tab` briefly shows the overlay, releasing switches to the target window
- Manual mode: overlay stays open until `Enter` is pressed

### Shortcuts

| Action                            | Default Keys               |
| --------------------------------- | -------------------------- |
| Switch to next (auto-confirm)     | `Alt + Tab`                |
| Switch to previous (auto-confirm) | `Alt + Shift + Tab`        |
| Switch to next (manual)           | `Alt + Ctrl + Tab`         |
| Switch to previous (manual)       | `Alt + Ctrl + Shift + Tab` |

---

## Workspaces Viewer

ID: `@seelen/workspaces-viewer`\
Trigger: `Win + Tab`

A full-screen virtual desktop overview, replacing the Windows Task View.

### Features

- Shows all monitors with their virtual workspaces
- Each workspace displays thumbnails of open windows
- Click a workspace to switch to it
- ESC or click outside to close

### Shortcuts

| Action                   | Default Keys |
| ------------------------ | ------------ |
| Toggle Workspaces Viewer | `Win + Tab`  |

### Virtual Desktop Shortcuts (System-Level)

These shortcuts are available even when the widget is not open:

| Category                     | Actions                                    |
| ---------------------------- | ------------------------------------------ |
| Main VD (`vdMain`)           | Create, destroy, rename workspaces         |
| Switch VD (`vdSwitch`)       | Jump to workspace 1–N                      |
| Move window to VD (`vdMove`) | Send focused window to workspace 1–N       |
| Send and follow (`vdSend`)   | Move window and switch to target workspace |

---

## Popup Widget

ID: `@seelen/popup`\
Preset: `Popup`\
Hidden: `true` (not shown in Settings navigation)\
Instances: `Multiple`

A programmatic popup widget for displaying arbitrary structured content. Used internally by Seelen UI tooling and
available to third-party integrations.

### Content Types

Popup content is composed of typed blocks:

| Type     | Description                                     |
| -------- | ----------------------------------------------- |
| `text`   | Plain or formatted text block                   |
| `icon`   | Icon element (supports standard icon libraries) |
| `image`  | Image block (URL or base64)                     |
| `button` | Clickable action button                         |
| `group`  | Container that groups other content blocks      |

### Popup Structure

A popup is built from three optional zones:

| Zone      | Description                                                |
| --------- | ---------------------------------------------------------- |
| `title`   | Header content (typically a text or icon+text combination) |
| `content` | Main body (one or more typed blocks)                       |
| `footer`  | Action area (typically buttons)                            |

### Behavior

- Multiple popup instances can exist simultaneously (one per invocation)
- Each popup is positioned relative to the trigger element or screen coordinates
- Focus loss closes the popup automatically (Popup preset behavior)
- Lazy-loaded: only created on first invocation, destroyed after idle timeout

---

## Quick Settings

ID: `@seelen/quick-settings`\
Triggered from the toolbar (installed as a plugin)

A compact popup panel for quick system toggles and controls.

### Controls

| Control            | Description                                                              |
| ------------------ | ------------------------------------------------------------------------ |
| Radio Buttons      | Toggle individual radio adapters: Wi-Fi, Bluetooth, Mobile Broadband, FM |
| Brightness Control | Adjust monitor brightness via DDC/CI (supports hardware dimming)         |
| Media Devices      | Volume control and output device selection                               |
| Settings button    | Opens Seelen UI Settings window                                          |
| Power button       | Opens Power Menu                                                         |

---

## Notifications

ID: `@seelen/notifications`\
Triggered from the toolbar (installed as a plugin)

A notification center popup that collects all Windows toast notifications.

### Features

- List all pending notifications with full content (title, body, images, actions)
- Dismiss individual notifications
- Clear all notifications at once
- Do Not Disturb (DND) toggle — switches `NotificationsMode` between `All` and `AlarmsOnly`
- Link to Windows Notification Settings
- Supports rich toast content: text, images, action buttons, progress bars

---

## Clipboard History

A Windows clipboard history backend built on the WinRT `SystemClipboardHistory` API. This is a pure backend feature
exposed to third-party widgets and toolbar plugins via IPC commands — there is no dedicated Seelen UI widget bundled for
it, but all APIs are public.

### Clipboard Entry Structure

| Field           | Description                                                 |
| --------------- | ----------------------------------------------------------- |
| `id`            | Unique entry identifier                                     |
| `timestamp`     | ISO 8601 timestamp of when the entry was added              |
| `sourceAppName` | Display name of the application that wrote to the clipboard |
| `sourceAppLogo` | Base64-encoded logo of the source application               |
| `content`       | `ClipboardEntryContent` (see below)                         |

### Clipboard Entry Content Types

| Field             | Description                                    |
| ----------------- | ---------------------------------------------- |
| `text`            | Plain text value                               |
| `html`            | HTML-formatted text                            |
| `rtf`             | Rich Text Format content                       |
| `applicationLink` | Protocol link for application-specific content |
| `webLink`         | HTTP/HTTPS URL                                 |
| `bitmap`          | Image content encoded as base64 WebP           |
| `files`           | Array of file paths                            |

Multiple content fields can be populated simultaneously (e.g. an entry may have both `text` and `html`).

### Available Commands

| Command                      | Description                                       |
| ---------------------------- | ------------------------------------------------- |
| `get_clipboard_history`      | Returns the full list of `ClipboardEntry` objects |
| `delete_clipboard_entry(id)` | Remove a single entry from the history by ID      |
| `clear_clipboard_history`    | Wipe the entire clipboard history                 |
| `set_clipboard_content(id)`  | Make an entry the current clipboard contents      |

### Events

| Event                            | Description                                                 |
| -------------------------------- | ----------------------------------------------------------- |
| `ClipboardHistoryChanged`        | Fires when entries are added, removed, or cleared           |
| `ClipboardHistoryEnabledChanged` | Fires when the Windows clipboard history feature is toggled |

### Implementation Notes

- Runs on a dedicated STA (Single-Threaded Apartment) thread required by WinRT clipboard APIs
- Automatically registers and unregisters WinRT event listeners with proper cleanup
- Processes all clipboard formats: text, HTML, RTF, images (converted to WebP), file drops, and protocol links

---

## Flyouts

ID: `@seelen/flyouts`

An ambient overlay that appears automatically when certain system events occur, showing a brief contextual notification.

### Trigger Events (configurable)

| Setting                 | Default | Trigger                                   |
| ----------------------- | ------- | ----------------------------------------- |
| `showVolumeChange`      | `true`  | Volume level changed                      |
| `showBrightnessChange`  | `true`  | Monitor brightness changed                |
| `showMediaPlayerChange` | `true`  | Track changed or playback started/stopped |
| `showWorkspaceChange`   | `true`  | Active virtual desktop changed            |
| `showNotifications`     | `true`  | New Windows notification arrived          |

### Settings

| Setting      | Options / Range                                                                                       |
| ------------ | ----------------------------------------------------------------------------------------------------- |
| `placement`  | Corner/edge: `top-left`, `top`, `top-right`, `left`, `right`, `bottom-left`, `bottom`, `bottom-right` |
| `margin`     | Distance from screen edge in px (default 30)                                                          |
| `timeToShow` | Display duration in seconds (1–10, default 4)                                                         |

### Flyout Content

Depending on the trigger:

- **Volume** — current output device name + volume level + icon
- **Brightness** — brightness level bar
- **Media** — track title, artist, album art
- **Workspace** — workspace name/index
- **Notification** — full notification toast preview

---

## Media Popup

ID: `@seelen/media-popup`\
Triggered from the toolbar (installed as a plugin)

A popup for audio management.

### Features

- **Main View**:
  - Global volume slider for the default output device
  - Per-session volume controls (individual app volumes)
  - Active media players list (title, artist, play/pause, next/prev)
  - Click on a media player to expand it

- **Device View** (accessible per-device):
  - Switch between audio output devices
  - Set default multimedia device
  - Mute/unmute

---

## Network Popup

ID: `@seelen/network-popup`\
Triggered from the toolbar (installed as a plugin)

Wi-Fi management popup.

### Features

- Wi-Fi radio toggle (on/off)
- **Connected** section — current network with signal strength
- **Saved** (known) networks list with auto-connect
- **Available** networks scan with real-time scanning indicator
- Connect to a network (with password prompt for secured networks)
- Forget a saved network
- Signal strength indicator per network
- Link to Windows Network Settings

---

## Bluetooth Popup

ID: `@seelen/bluetooth-popup`\
Triggered from the toolbar (installed as a plugin)

Bluetooth device management popup.

### Features

- Bluetooth radio toggle (on/off)
- **Connected** devices section (paired + currently connected)
- **Paired** devices (paired but not connected)
- **Available** devices section with real-time scan indicator
- Connect / disconnect devices
- Pair new devices (with PIN display/confirmation when required)
- Forget a device
- Duplicate device deduplication (classic vs. LE versions of the same device)
- Link to Windows Bluetooth Settings

---

## Calendar Popup

ID: `@seelen/calendar-popup`\
Triggered from the toolbar (installed as a plugin)

A compact calendar popup showing the current date.

### Features

- Full monthly calendar grid
- `startOfWeek` from global settings (Monday / Sunday / Saturday)
- Current day highlighted

---

## Power Menu

ID: `@seelen/power-menu`\
Triggered from the toolbar plugin, Start Menu footer, or Quick Settings footer.

A full-screen overlay with system power/session actions.

### Power Options

| Option    | Action                       |
| --------- | ---------------------------- |
| Lock      | Lock the Windows session     |
| Log Out   | Sign out of the current user |
| Shutdown  | Power off the computer       |
| Reboot    | Restart the computer         |
| Suspend   | Put the computer to sleep    |
| Hibernate | Hibernate the computer       |

### Display

- Shows current user's profile picture
- Shows user display name / email
- ESC or click outside to close
- DPI-aware positioning aligned to the primary monitor

---

## User Menu

ID: `@seelen/user-menu`\
Triggered from the Start Menu footer or toolbar plugin.

User profile and quick file access popup.

### Features

- User profile picture and display name
- **Display name source** (configurable):
  - Windows profile name
  - Xbox Gamertag
- Quick access to known Windows user folders:
  - Documents, Downloads, Music, Pictures, Videos, Desktop, etc.
- File preview on hover
- Open Seelen UI installation folder
- Open Seelen UI log folder

---

## System Tray

ID: `@seelen/system-tray`\
Toolbar plugin that renders the system tray icon area.

### Features

- Displays all Windows system tray notification icons
- Left/right click actions forwarded to the native tray icon
- Expand/collapse to show hidden icons
- Hover tooltips from the native tray icon

---

## Keyboard Selector

ID: `@seelen/keyboard-selector`\
Triggered from toolbar plugin.

A popup for switching the active keyboard layout.

### Features

- Lists all installed keyboard input methods
- Shows active layout highlighted
- Click to switch the active layout immediately
- IME (Input Method Editor) status awareness

---

## Context Menu

ID: `@seelen/context-menu`

A custom right-click context menu rendered by Seelen UI widgets.

### Features

- Hierarchical menu items with submenus
- Icon support per item
- Separator lines
- Keyboard navigation

---

## Tooltip

ID: `@seelen/tooltip`

A shared tooltip popup used by toolbar items.

- Renders HTML content returned by the item's `tooltip` JS function
- Auto-positioned relative to the triggering element

---

## Settings Window

ID: `@seelen/settings`\
Shortcut: configurable (default shown in shortcuts panel)

The main configuration UI for Seelen UI.

### Sections

| Section           | Description                                                                                                   |
| ----------------- | ------------------------------------------------------------------------------------------------------------- |
| Home              | News, mini store, quick links                                                                                 |
| General           | Language, date format, autostart, accent color, hardware acceleration, polling interval, suspend on game mode |
| Performance       | Performance mode profiles (plugged / on battery / energy saver)                                               |
| SeelenWeg         | Dock configuration                                                                                            |
| Fancy Toolbar     | Toolbar layout and hide settings                                                                              |
| Window Manager    | Tiling WM settings, layouts, border, animations, drag behavior                                                |
| Wallpaper Manager | Wallpaper engine settings, collections                                                                        |
| Apps Menu         | (via widget settings)                                                                                         |
| Shortcuts         | Global shortcut manager with key remapping                                                                    |
| By Monitor        | Per-monitor widget and wallpaper overrides                                                                    |
| App Rules         | Per-application behavior overrides                                                                            |
| Themes            | Theme browser, activation, and variable customization                                                         |
| Wallpapers        | Wallpaper browser, thumbnails, collection management                                                          |
| Widgets           | Widget browser and per-widget configuration                                                                   |
| Icon Packs        | Icon pack browser and editor                                                                                  |
| Plugins           | Toolbar plugin browser                                                                                        |
| Sound Packs       | Sound effect pack browser (placeholder — not yet implemented)                                                 |
| Extras            | Version info, update channel, Discord RPC, backup sync, streaming mode, cache management, restart/exit        |
| Developer         | Exposed only when `devTools: true`; raw JSON editor and API testing                                           |

---

## Plugins System

Plugins extend widget functionality. A plugin has:

- **ID** — unique resource ID
- **Target** — the widget it attaches to
- **Type** — `Known` (built-in integration) or `ThirdParty` (custom)
- **Metadata** — display name, description, icon

### Known Plugin Kinds

| Kind                  | Target                   | Purpose                                  |
| --------------------- | ------------------------ | ---------------------------------------- |
| Window Manager Layout | `@seelen/window-manager` | Defines a tiling layout structure        |
| Toolbar Item          | `@seelen/fancy-toolbar`  | Adds a configured item to the toolbar    |
| Context Menu Item     | Any widget               | Adds an entry to the widget context menu |

### Third-Party Plugin (Toolbar)

Custom toolbar plugins can define JS `template`, `tooltip`, `badge`, and `onClick` functions along with scopes and
remote data declarations — the same interface as built-in toolbar items.

---

## Per-Monitor Configuration

Each connected physical monitor can have independent settings.

### Monitor-Level Overrides

| Override             | Description                                                        |
| -------------------- | ------------------------------------------------------------------ |
| Wallpaper Collection | Which collection to use on this monitor (overrides global default) |
| Per-widget settings  | Any widget setting that declares `allowSetByMonitor: true`         |

### Per-Workspace Overrides

Each virtual workspace on a monitor can also override:

| Override             | Description                                   |
| -------------------- | --------------------------------------------- |
| Wallpaper Collection | Which collection to display on this workspace |

### Widget ReplicaByMonitor

Widgets with `instances: ReplicaByMonitor` (Weg, Toolbar, Window Manager) run one instance per monitor and receive a
`monitorId` in their URL. Monitor-specific settings are merged on top of global settings.

---

## App Rules (Per-App Configuration)

Applications can be individually configured to modify their behavior within Seelen UI.

### Matching an App

Apps are matched using an `AppIdentifier`:

| Field              | Description                                                |
| ------------------ | ---------------------------------------------------------- |
| `kind`             | Match by: `Exe`, `Class`, `Title`, or `Path`               |
| `id`               | The value to match                                         |
| `matchingStrategy` | `Equals`, `StartsWith`, `EndsWith`, `Contains`, or `Regex` |
| `negation`         | Invert the match                                           |
| `and` / `or`       | Compound conditions                                        |

### App Rule Properties

| Property         | Description                            |
| ---------------- | -------------------------------------- |
| `name`           | Human-readable label                   |
| `category`       | Grouping category                      |
| `boundMonitor`   | Force app to open on monitor index N   |
| `boundWorkspace` | Force app to open on workspace index N |
| `options`        | Array of behavior flags (see below)    |

### App Extra Flags

| Flag            | Effect                                                          |
| --------------- | --------------------------------------------------------------- |
| `NoInteractive` | Window receives no mouse/keyboard input                         |
| `WmFloat`       | Always float in the tiling WM (never tiled)                     |
| `WmForce`       | Force the WM to manage this window even if it normally wouldn't |
| `WmUnmanage`    | Exclude from WM management entirely                             |
| `VdPinned`      | Pin the app to all virtual desktops (always visible)            |

### Bundled App Rules

Seelen UI ships with a curated set of bundled rules for common system applications (e.g. dialog boxes, system utilities)
that are automatically excluded or floated by the WM. These are read-only; users can override them by creating a
duplicate rule.

---

## Shortcuts

All keyboard shortcuts in Seelen UI are user-remappable. The shortcut system has two layers:

### System Shortcuts

Global shortcuts registered at the OS level, not tied to a specific widget:

| Group                           | Examples                           |
| ------------------------------- | ---------------------------------- |
| Virtual Desktop — Main          | Create/destroy/rename workspaces   |
| Virtual Desktop — Switch        | Jump to workspace N                |
| Virtual Desktop — Move          | Move focused window to workspace N |
| Virtual Desktop — Send & Follow | Move + switch to target workspace  |
| Misc                            | Miscellaneous system shortcuts     |

### Widget Shortcuts

Each widget declares its own shortcuts. They are grouped by widget in the Shortcuts settings panel:

| Widget            | Shortcut IDs                                                          |
| ----------------- | --------------------------------------------------------------------- |
| SeelenWeg         | `weg-launch-0` … `weg-launch-9`                                       |
| Window Manager    | All WM movement, resize, and mode shortcuts                           |
| Wallpaper Manager | `wallpaper-next`, `wallpaper-prev`                                    |
| Task Switcher     | `task-switcher-next-auto`, `task-switcher-prev-auto`, manual variants |
| Apps Menu         | `apps-menu-toggle` (Win key — system, read-only)                      |
| Workspaces Viewer | `vd-toggle-view` (Win+Tab — system, read-only)                        |
| Settings          | `settings-open`                                                       |

### Shortcut Settings

| Setting     | Description                                      |
| ----------- | ------------------------------------------------ |
| `enabled`   | Master on/off for all shortcuts                  |
| `shortcuts` | Map of `shortcut-id → [key array]` for overrides |

Read-only shortcuts (marked `readonly: true`) cannot be remapped in the UI.\
System shortcuts (marked `system: true`) intercept the OS-native handler (e.g. `Win` key).

---

## Performance Mode

Three performance levels configurable per power state:

| Mode       | Behavior                                                       |
| ---------- | -------------------------------------------------------------- |
| `Disabled` | No performance restrictions; full animations and CSS effects   |
| `Minimal`  | Disables CSS animations and non-essential transitions          |
| `Extreme`  | Disables all CSS animations and may suspend background renders |

Power states:

- **Plugged In** — default mode
- **On Battery** — mode when running on battery power
- **Energy Saver** — mode when Windows Energy Saver is active

---

## Backup & Sync

| Feature             | Description                                           |
| ------------------- | ----------------------------------------------------- |
| `backupSyncEnabled` | Automatic cloud synchronization of Seelen UI settings |
| Last sync timestamp | Shown in Settings → Extras                            |
| Session-gated       | Requires a logged-in Seelen UI account                |

---

## Developer & Extras

### Extras Settings

| Setting          | Description                                          |
| ---------------- | ---------------------------------------------------- |
| Version display  | Shows app version, build type (dev / msix / fixed)   |
| Update Channel   | Select `Stable`, `Beta`, or `Nightly`                |
| Discord RPC      | Show current Seelen UI status in Discord             |
| Streaming Mode   | Hide personal info (email, names) in visible content |
| Backup Sync      | Toggle cloud sync (requires session)                 |
| Clear Icon Cache | Force-regenerate all extracted app icons             |
| Relaunch         | Restart Seelen UI process                            |
| Exit             | Terminate Seelen UI                                  |

### Developer Tools

Enabled by setting `devTools: true` in Settings → General.

- Exposes a **Developer** tab in Settings
- Enables WebView DevTools on right-click in any widget
- Adds "End Task" to dock context menus
- Raw JSON settings editor
- Simulation of permission states

### Session / Authentication

- Users can log into a Seelen UI cloud account
- Authentication tokens are stored in the Windows Credential Manager (never exposed to widgets)
- Required for Backup & Sync features
- Profile name / Xbox Gamertag displayed in the User Menu

---

## Architecture Notes

### Widget Presets

Every widget has a `preset` that configures its default behavior:

| Preset    | Behavior                                       |
| --------- | ---------------------------------------------- |
| `None`    | No automatic behavior                          |
| `Desktop` | Desktop-level window; position/size persisted  |
| `Overlay` | Floats above all windows                       |
| `Popup`   | Shows at trigger position; hides on focus loss |

### Widget Instances

| Mode               | Behavior                                |
| ------------------ | --------------------------------------- |
| `Single`           | One instance shared across all monitors |
| `ReplicaByMonitor` | One instance per connected monitor      |
| `Multiple`         | Unlimited simultaneous instances        |

### Lazy Widgets

Widgets with `lazy: true` are not created until first triggered. After being hidden, they are destroyed after a
30-second idle timeout to reclaim memory.

### Widget Config Hierarchy

Settings for a widget are merged in this priority order (highest wins):

1. Monitor-specific overrides (`monitorsV3[monitorId].byWidget[widgetId]`)
2. Instance-specific overrides (`byWidget[widgetId].$instances[instanceId]`)
3. Global widget settings (`byWidget[widgetId]`)
4. Widget default values (declared in `metadata.yml`)
