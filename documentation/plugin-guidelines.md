# Seelen UI — Plugin Guidelines

Reference guide for creating plugins. A plugin is a declaration file that extends the functionality of a specific
widget. The plugin itself only declares who it targets and provides a data payload — the target widget is entirely
responsible for loading, parsing, and deciding what to do with that data.

Read [resource guidelines](./resource-guidelines) first for concepts shared across all resource kinds.

---

## Table of Contents

1. [What is a Plugin?](#1-what-is-a-plugin)
2. [Plugin Structure — metadata.yml](#2-plugin-structure--metadatayml)
3. [Plugin Data](#3-plugin-data)

---

## 1. What is a Plugin?

A plugin targets a widget by ID and provides it with arbitrary data. The widget discovers installed plugins at runtime,
reads the ones that target it, and uses the data however it sees fit. A plugin has no logic of its own — it is a pure
declaration.

**Plugins are merely plain, flat files — nothing more.** The `Plugin` resource kind does not define a runtime, a
sandbox, or an execution model of any kind. It is only an envelope: `id`, `target`, and a free-form `plugin` payload.
There is no such thing as "the plugin system" in the singular — there is one plugin **envelope** shared by every
resource, and as many plugin **behaviors** as there are widgets willing to consume one. Seelen UI itself does not parse,
validate, or execute the contents of `plugin` in any generic way; it only routes the resource to whichever widget's
`target` matches.

Think of plugins as **per-widget extensions**: each target widget defines its own tiny "extension API" — its own schema
for what `plugin` must contain, and its own logic for parsing and executing that data (JS `eval` in a sandbox, a
declarative tree it walks, a lookup table it renders from, anything the widget author wants). A toolbar plugin might
define a new button with JS callbacks; a tiling window manager plugin might define a static layout tree with no code at
all; a calendar widget's plugin might add a new event source. **The schema, the parsing, and the execution all live in
the target widget — not in the plugin resource, and not in Seelen UI's core.**

For Seelen UI's own built-in widgets that accept plugins, see the dedicated guides:

- [Toolbar Plugins](./toolbar-plugins) — `@seelen/fancy-toolbar`
- [Dock Plugins](./dock-plugins) — `@seelen/weg`
- [Window Manager Layouts](./wm-layouts) — `@seelen/window-manager`

If you are building your own widget and want to support plugins, you are free to design any `plugin` schema you like —
document it for your users the same way the three guides above document Seelen UI's built-in widgets.

---

## 2. Plugin Structure — metadata.yml

```yaml
id: "@yourname/my-plugin"

metadata:
  displayName: My Plugin
  description: A short description of what this plugin adds.
  tags:
    - toolbar

# Icon shown in the Seelen UI settings panel.
# Must be a valid react-icons name (https://react-icons.github.io/react-icons/).
# Defaults to PiPuzzlePieceDuotone if omitted.
icon: PiPuzzlePieceDuotone

# The widget this plugin is for
target: "@someuser/some-widget"

# The plugin data — structure depends entirely on the target widget
plugin:
  someField: someValue
```

The only required fields are `id`, `target`, and `plugin`.

---

## 3. Plugin Data

The `plugin` field is free-form — it can be any valid YAML value (a map, a list, a string, anything). There is no schema
enforced at the resource level. The target widget receives it as-is and is responsible for validating and using it.

Read the documentation of the widget you are targeting to know what structure it expects.

You can use `!include` in the `plugin` block just like anywhere else in a resource file:

```yaml
plugin:
  template: !include plugin/template.js
  tooltip: !include plugin/tooltip.js
  scopes:
    - Power
```
