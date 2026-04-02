# Seelen UI — Plugin Guidelines

Reference guide for creating plugins. A plugin is a declaration file that extends the functionality of a specific
widget. The plugin itself only declares who it targets and provides a data payload — the target widget is entirely
responsible for loading, parsing, and deciding what to do with that data.

Read [resource_guidelines](./resource_guidelines) first for concepts shared across all resource kinds.

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

What a plugin _means_ depends entirely on the widget it targets. A toolbar plugin might define a new button. A calendar
widget plugin might add a new event source. Whatever schema the widget expects is what the plugin must provide.

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
