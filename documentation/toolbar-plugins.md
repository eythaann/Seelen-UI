# Toolbar Plugins — `@seelen/fancy-toolbar`

This is one concrete example of the generic Plugin mechanism described in [plugin guidelines](./plugin-guidelines):
`@seelen/fancy-toolbar` is the target widget, and this page documents **its** schema for `plugin`, and **its** rules for
parsing and executing that data. None of this is special-cased in Seelen UI's core — the toolbar widget owns all of it.

---

## 1. The `plugin` Payload — `ToolbarItem`

```yaml title="src/static/widgets/notifications/toolbar-plugin.yml"
id: "@seelen/tb-notifications"
target: "@seelen/fancy-toolbar"
plugin:
  scopes:
    - Notifications
  template: >-
    return [
      dndActive ? icon("TbZzz") : null,
      count > 0 ? icon("MdNotificationsActive") : icon("MdOutlineNotifications"),
    ]
  badge: "return count > 0 ? count : null"
  tooltip: 'return [t("placeholder.notifications"), ": ", count]'
  onClickV2: |-
    trigger("@seelen/notifications");
```

The full field set (`scopes`, `template`, `render`, `canvasSize`, `tooltip`, `badge`, `onClick`/`onClickV2`,
`onWheelUp`, `onWheelDown`, `style`, `remoteData`) is shown in the sections below.

Every field except `scopes` and `template` is optional. `template`, `tooltip`, `badge`, `onClick`, `onWheelUp`, and
`onWheelDown` are all **JS function bodies** written as plain strings (typically pulled in with `!include` from a `.js`
file) — not JSON, not a declarative object. The toolbar widget compiles and runs each one in a sandbox.

> Legacy alias: `onClickV2` is accepted as an alternate key for `onClick` — several bundled plugins use it, both work
> identically.

---

## 2. `scopes` — What Data Gets Injected

Each entry in `scopes` (case-insensitive) makes a set of fields available inside `template`/`tooltip`/`badge`/ `onClick`
as top-level variables — no `scope.` prefix needed.

| Scope               | Injected variables                                                                                              | Backing command(s)                                                               |
| ------------------- | --------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| `Date`              | `date` (formatted string)                                                                                       | local reactive clock                                                             |
| `Notifications`     | `count`, `dndActive`                                                                                            | `GetNotifications`, `GetNotificationsMode`                                       |
| `Media`             | `defaultOutputDevice`, `defaultInputDevice`, `volume`, `isMuted`, `inputVolume`, `inputIsMuted`, `mediaSession` | `GetMediaSessions`, `GetMediaDevices`                                            |
| `Network`           | `online`, `interfaces` (`NetworkAdapter[]`), `usingInterface`                                                   | `GetNetworkInternetConnection`, `GetNetworkAdapters`, `GetNetworkDefaultLocalIp` |
| `Keyboard`          | `activeLang`, `activeKeyboard`, `activeLangPrefix`, `activeKeyboardPrefix`, `languages`, `imeState`             | `SystemGetLanguages`, `SystemGetImeState`                                        |
| `User`              | `user` (includes computed `displayName`)                                                                        | `GetUser`                                                                        |
| `Bluetooth`         | `devices`, `getIconNameForBTDevice(device)`                                                                     | `GetBluetoothDevices`                                                            |
| `Power`             | `power` (`PowerStatus`), `powerMode`, `batteries` (`Battery[]`)                                                 | `GetPowerStatus`, `GetPowerMode`, `GetBatteries`                                 |
| `FocusedApp`        | `focusedApp`                                                                                                    | `GetFocusedApp`                                                                  |
| `Workspaces`        | `workspaces`, `activeWorkspace`                                                                                 | `StateGetVirtualDesktops`                                                        |
| `Disk`              | `disks` (`Disk[]`)                                                                                              | `GetSystemDisks`                                                                 |
| `NetworkStatistics` | `networkStatistics`                                                                                             | `GetSystemNetwork`                                                               |
| `Memory`            | `memory` (`Memory`)                                                                                             | `GetSystemMemory`                                                                |
| `Cpu`               | `cores` (`Core[]`)                                                                                              | `GetSystemCores`                                                                 |
| `Tray`              | `trayIcons`                                                                                                     | `GetSystemTrayIcons`                                                             |
| `SessionTime`       | `activeSessionTime` (`{ activeSeconds, longestActiveSeconds, isIdle }`)                                           | `GetActiveSessionTime`                                                           |
| `TrashBin`          | `trashBinInfo` (`TrashBinInfo`)                                                                                 | `GetTrashBinInfo`                                                                |
| `Waveform`          | `waveform` (`AudioWaveform`)                                                                                    | `GetMediaWaveform`                                                               |

The built-in `@default/session-time` toolbar plugin displays active screen time for the current Windows login. It tracks
only while the module is present in the toolbar; removing it pauses tracking, and re-adding it resumes from the saved
timer and longest-session record. It does not count sleep, and pauses after five minutes without input unless a media
session is playing. While paused, it shows
`Zzz`; clicking it opens the longest-session score, which updates live when the current session beats the previous
record. The longest duration and current timer checkpoint are written once per second to
`%LOCALAPPDATA%\com.seelen.seelen-ui\active-session-time.log`, outside the Seelen installation. A Seelen-only restart
restores the current timer without counting the time Seelen was stopped; a Windows reboot starts a fresh current timer
while retaining the longest-session record. Use the small tools icon below the timer to enable periodic active-time
alerts, set an interval such as `1h30m`, and choose a Windows system sound or an MP3 file. The alert settings include a
test button.

The built-in `@default/weather` toolbar module is opt-in by being present in the toolbar. It keeps a cloud icon visible
while weather is loading or before a location is available. By default it requests approximate coordinates through the
system geolocation API, then uses Open-Meteo for forecast data; Windows location permission is required. Windows has no
supported, universal local weather-forecast API, so the popup offers manual city or postal-code entry if system location
is unavailable. Location names are hidden from the taskbar by default, and no location or forecast request is made
while the module has no active consumer. Forecasts refresh every 15 minutes only while the module or its popup is active;
the request contains current conditions, one day of summaries, and only the six displayed hourly points. Removing the
module from the toolbar stops refresh requests. Taskbar details default to temperature and precipitation chance, with
settings for the other fields, showing the location name, regional number/date formatting, and temperature, wind, and
precipitation units. Moon phase and illumination are calculated locally.

Shapes of the more structured values (generated TS, `libs/core/gen/types/`):

```ts
type Memory = { total: number; free: number; swapTotal: number; swapFree: number };
type Core = { name: string; brand: string; usage: number; frequency: number };
type Disk = {
  name: string;
  fileSystem: string;
  totalSpace: number;
  availableSpace: number;
  mountPoint: string;
  isRemovable: boolean;
  readBytes: number;
  writtenBytes: number;
};
type PowerStatus = {
  acLineStatus: string;
  batteryFlag: string;
  batteryLifePercent: number;
  systemStatusFlag: string;
  batteryLifeTime: number;
  batteryFullLifeTime: number;
};
type Battery = {
  vendor: string;
  model: string;
  serialNumber: string;
  technology: string;
  state: string;
  capacity: number;
  temperature: number;
  percentage: number;
  cycleCount: number;
  smartCharging: boolean;
  energy: number;
  energyFull: number;
  energyFullDesign: number;
  energyRate: number;
  voltage: number;
  timeToFull: number;
  timeToEmpty: number;
};
type NetworkAdapter = {
  name: string;
  description: string;
  status: string;
  dnsSuffix: string;
  type: string;
  ipv6: string[];
  ipv4: string[];
  gateway: string | null;
  mac: string | null;
};
type TrashBinInfo = { itemCount: number; sizeInBytes: number };
```

---

## 3. `remoteData` — Fetching External Data Into Scope

```yaml
remoteData:
  weather:
    url: "https://api.example.com/weather"
    requestInit: null
    updateIntervalSeconds: 600
```

For each key, the toolbar `fetch()`es `url` (with the given `requestInit`, if any), parses the response as JSON or text
depending on `Content-Type`, and injects the result under that key into the script scope — so the example above makes a
`weather` variable available inside `template`. If `updateIntervalSeconds` is set, the fetch repeats on that interval.

---

## 4. Execution Model — What Each Script Can Do

All scripts run inside a JS sandbox (`@nyariv/sandboxjs`), not raw `eval`. There are two categories of script with two
different scopes:

### Content scripts — `template`, `tooltip`, `badge`

Run with `{ ...resolvedScopes, ...resolvedRemoteData, t }`, where `t(key, args)` is the i18n lookup function. You must
`return` a value — the return value becomes the rendered content:

- Strings, numbers, and booleans are stringified directly.
- Arrays are flattened and each item rendered in sequence.
- Special tagged objects render as real UI elements, produced by helper functions available in scope:
  - `icon(name, size)` / `Icon(...)` — an icon from the bundled icon set
  - `AppIcon(...)` — an application icon
  - `Image({ path | url, version? })` — an image element; pass `version` (any string or number that changes with the
    file contents) to force a reload when the file is overwritten in place
  - `Button(...)` — a clickable button element
  - `Group(...)` — a container grouping other elements

```js title="src/static/plugins/tb_cpu_usage/plugin/template.js"
const totalUsage = cores.reduce((total, core) => total + core.usage, 0);
const used = totalUsage / cores.length;
return [icon("LuCpu"), " ", used.toFixed(0) + "%"];
```

### Canvas script — `render`

`render` is an alternative to `template` for cases where a plugin needs pixel-level control (custom icons, waveforms,
sparklines, etc.) instead of composing `icon()`/`Image()`/`Button()`/`Group()` elements. If a plugin sets `render`, it
takes priority over `template` for the item's main content — `template` is ignored in that case. `tooltip`, `badge`, and
the action scripts are unaffected and keep working exactly as described elsewhere on this page.

Unlike the dock (see [Dock Plugins — §2](./dock-plugins#2-render)), toolbar items have no fixed size — width is normally
intrinsic to content — so a `render`-based item needs an explicit width to give the canvas something to draw into:

- `canvasSize` (`number`, optional) — width in **pixels** of the canvas. If omitted, the item is drawn square (width =
  height).
- The canvas height always tracks the toolbar's configured item size (`--config-item-size`), the same value every other
  toolbar item's height derives from.

The scope contract for `render` is otherwise identical to the dock's: it runs with the resolved `scopes`/`remoteData`
plus `isDarkMode`, `systemTokens`/`themeTokens` (CSS custom-property color values as plain strings), and a `canvas`
object (`{ getContext(contextId), width, height }`). See
[Dock Plugins — §2.1](./dock-plugins#21-drawing-onto-the-canvas-default) for the full drawing API and an example that
draws with `ctx.arc/lineTo/stroke/fill`.

```yaml
plugin:
  scopes: [Waveform]
  render: !include render.js
  canvasSize: 40
```

### Action scripts — `onClick`, `onWheelUp`, `onWheelDown`

Run with `{ ...resolvedScopes, ...resolvedRemoteData, invoke, open, trigger }`. Return value is ignored — these are
fire-and-forget handlers.

- `invoke(command, args)` — **whitelisted**: only a small allow-list of `SeelenCommand`s can be called this way (e.g.
  `SwitchWorkspace`, `SetVolumeLevel`, `OpenFile`). This is not the full command surface — it exists so toolbar items
  can trigger a handful of safe system actions without a full IPC bridge.
- `open(path)` — opens a file, URL, or `ms-settings:` shell URI with the OS default handler.
- `trigger(widgetId)` — pops up another widget (typically a `Popup`-preset widget like `@seelen/bluetooth-popup`)
  anchored at this toolbar item's screen position. This is how toolbar items open their associated popups.

```yaml title="src/static/widgets/bluetooth-popup/toolbar-plugin.yml"
plugin:
  scopes: [Bluetooth]
  tooltip: |-
    return "Bluetooth";
  template: !include plugin.js
  onClickV2: |-
    trigger("@seelen/bluetooth-popup");
```

Using `open()`:

```yaml title="src/static/plugins/tb_default_power/metadata.yml"
plugin:
  scopes: [Power]
  tooltip: !include plugin/tooltip.js
  template: !include plugin/template.js
  onClickV2: open("ms-settings:powersleep")
```

---

## 5. `style`

A plain object merged onto the item's container as inline styles, keyed like a React `style` prop
(`{ "margin-left": "4px" }`, numbers allowed for unitless properties).

---

## 6. Referencing an Installed Plugin vs. Writing Inline

In the Fancy Toolbar's own settings, a toolbar item slot accepts either an inline `ToolbarItem` object (as shown above)
or a plain string — the `id` of an already-installed `Plugin` resource targeting `@seelen/fancy-toolbar`. Both forms
resolve to the exact same `ToolbarItem` shape by the time it reaches the execution engine described in section 4.

### Weather defaults

The weather plugin defaults to imperial units (�F, mph, inches, inHg). Its settings are stored in the Seelen settings file (`weatherSettings`) and persist across restarts. Screen-time can optionally show a toast notification when the sound timer fires.
