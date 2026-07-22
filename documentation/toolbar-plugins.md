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

The full field set (`scopes`, `template`, `tooltip`, `badge`, `onClick`/`onClickV2`, `onWheelUp`, `onWheelDown`,
`style`, `remoteData`) is shown in the sections below.

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
| `TrashBin`          | `trashBinInfo` (`TrashBinInfo`)                                                                                 | `GetTrashBinInfo`                                                                |
| `Waveform`          | `waveform` (`AudioWaveform`)                                                                                    | `GetMediaWaveform`                                                               |

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
  - `Image(...)` — an image element
  - `Button(...)` — a clickable button element
  - `Group(...)` — a container grouping other elements

```js title="src/static/plugins/tb_cpu_usage/plugin/template.js"
const totalUsage = cores.reduce((total, core) => total + core.usage, 0);
const used = totalUsage / cores.length;
return [icon("LuCpu"), " ", used.toFixed(0) + "%"];
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
