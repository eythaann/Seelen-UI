# Dock Plugins — `@seelen/weg`

This is one concrete example of the generic Plugin mechanism described in [plugin guidelines](./plugin-guidelines):
`@seelen/weg` (SeelenWeg, the dock/taskbar) is the target widget, and this page documents **its** schema for `plugin`,
and **its** rules for parsing and executing that data. None of this is special-cased in Seelen UI's core — the dock
widget owns all of it.

Note the distinction from pinned dock items: things like `AppOrFile` and `Separator` are the dock's **built-in,
config-driven item kinds** — they are not plugins. A **dock plugin** is a separate, distinct kind of dock item: a
reference to an installed `Plugin` resource targeting `@seelen/weg`. The bundled **Notifications** widget
(`@seelen/dock-notifications`, `src/static/widgets/notifications/dock-plugin.yml`) is a real, shipped example of this
extension point — used as the running example throughout this page.

---

## 1. The `plugin` Payload — `WegPluginItem`

```yaml title="src/static/widgets/notifications/dock-plugin.yml"
id: "@seelen/dock-notifications"
target: "@seelen/weg"
plugin:
  scopes:
    - Notifications
  render: !include dock_plugin.js
  badge: "return count > 0 ? count : null"
  tooltip: 'return "Notifications: " + count'
  onClick: |-
    trigger("@seelen/notifications");
```

`scopes` follow the same names and shapes documented in
[Toolbar Plugins — §2](./toolbar-plugins#2-scopes--what-data-gets-injected); the resolver is shared between the dock and
the toolbar. Here `Notifications` injects `count` and `dndActive` into every script below.

---

## 2. `render`

By default `render` draws onto a canvas — see §2.1. If the plugin sets `noCanvas: true`, `render` instead just `return`s
a string naming a custom icon key for the dock to look up, with no drawing involved — that's the whole mode, nothing
more to it.

### 2.1 Drawing onto the canvas (default)

`render` runs with the resolved `scopes` data plus `isDarkMode`, `systemTokens`/`themeTokens` (CSS custom-property color
values as plain strings), and a `canvas` object (`{ getContext(contextId), width: 256, height: 256 }`). It draws onto
`canvas.getContext("2d")` using the standard `CanvasRenderingContext2D` API — the dock exports the result via
`canvas.toBlob()` and displays it as the item's icon. Abridged for brevity:

```js title="src/static/widgets/notifications/dock_plugin.js"
const ctx = canvas.getContext("2d");
const color = themeTokens.foregroundColor;

// ...draws the bell shape using ctx.arc/lineTo/stroke/fill with `color`...

if (dndActive) {
  ctx.fillStyle = color;
  ctx.font = `bold ${Math.round(canvas.height * 0.3)}px sans-serif`;
  ctx.textAlign = "center";
  ctx.fillText("zZ", canvas.width / 2, canvas.height * 0.55);
}
```

---

## 3. `tooltip` / `badge`

Same base scope as `render`, expected to return a string (or, for `badge`, `null` to hide it).

```js
// tooltip — return "Notifications: " + count
// badge   — return count > 0 ? count : null
```

---

## 4. `onClick`

Executed with `{ ...scope, invoke, trigger }`.

- `invoke(command, args)` is **whitelisted** to a small allow-list of `SeelenCommand`s for the dock: `OpenFile`,
  `ShowDesktop`, `ShowStartMenu`. This is a different, narrower allow-list than the toolbar's — each target widget
  defines its own.
- `trigger(widgetId)` pops up another widget anchored at the item — this is how the Notifications dock plugin above
  opens the `@seelen/notifications` popup on click.

---

## 5. Context Menu Integration

If `scopes` includes `TrashBin` (case-insensitive), the dock automatically adds an "Empty Trash" entry to that item's
context menu — this is a dock-specific convenience tied to that one scope, not a general plugin feature.

---

## 6. Full Example — A Fictitious "Happy Face" Plugin

Everything above in one made-up, self-contained plugin: it draws a smiley on the canvas, shows a fake tooltip and a fake
badge count, and opens a folder when clicked. None of this data is real — it's here purely to show how every field from
the sections above fits together in one resource.

```
MyPlugin/
├── metadata.yml
├── render.js
├── tooltip.js
└── badge.js
```

```yaml title="MyPlugin/metadata.yml"
id: "@yourname/happy-dock"
metadata:
  displayName: Happy Face
  description: Shows a smiley in the dock. Purely decorative.
target: "@seelen/weg"
plugin:
  scopes: []
  render: !include render.js
  tooltip: !include tooltip.js
  badge: !include badge.js
  onClick: |-
    invoke(SeelenCommand.OpenFile, { path: "C:\\Users\\me\\Pictures" });
```

```js title="MyPlugin/render.js"
const ctx = canvas.getContext("2d");
const cx = canvas.width / 2;
const cy = canvas.height / 2;
const r = canvas.width * 0.4;

ctx.clearRect(0, 0, canvas.width, canvas.height);
ctx.strokeStyle = isDarkMode ? "#ffe066" : "#ffb703";
ctx.fillStyle = ctx.strokeStyle;
ctx.lineWidth = 10;

// face outline
ctx.beginPath();
ctx.arc(cx, cy, r, 0, Math.PI * 2);
ctx.stroke();

// eyes
ctx.beginPath();
ctx.arc(cx - r * 0.4, cy - r * 0.25, r * 0.08, 0, Math.PI * 2);
ctx.arc(cx + r * 0.4, cy - r * 0.25, r * 0.08, 0, Math.PI * 2);
ctx.fill();

// smile
ctx.beginPath();
ctx.arc(cx, cy, r * 0.5, 0.15 * Math.PI, 0.85 * Math.PI);
ctx.stroke();
```

```js title="MyPlugin/tooltip.js"
return "Smile! (this tooltip is fake)";
```

```js title="MyPlugin/badge.js"
// pretend there are always 3 unread "smiles"
return 3;
```

### Loading it with the `slu` CLI

While iterating, load the folder directly into a running Seelen UI instance — no bundling needed:

```bash
slu resource load plugin ./MyPlugin
```

The smiley shows up in the dock immediately. Edit any of the four files and re-run the same command to reload it. When
you're done, remove it with:

```bash
slu resource unload plugin ./MyPlugin
```

See [resource guidelines — §8](./resource-guidelines#8-loading-and-unloading-resources) for the full load/unload/bundle
workflow shared by every resource kind.
