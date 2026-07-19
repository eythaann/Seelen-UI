# Window Manager Layouts — `@seelen/window-manager`

This is one concrete example of the generic Plugin mechanism described in [plugin guidelines](./guidelines.md): the
tiling Window Manager (`@seelen/window-manager`) is the target widget, and this page documents **its** schema for
`plugin`, and **its** rules for parsing that data into a live layout tree. None of this is special-cased in Seelen UI's
core — the window manager owns all of it.

There is no separate "Layout" resource kind. A layout is simply a `Plugin` resource whose `target` is
`@seelen/window-manager`. Unlike toolbar and dock plugins, a WM layout has **no scripts at all** — it is a purely
declarative tree, walked by Rust code, not evaluated as JS.

---

## 1. The `plugin` Payload — `TwmPlugin`

```yaml
id: "@yourname/my-layout"
target: "@seelen/window-manager"
plugin:
  structure: # a TwmPluginNode tree, or omit/null for a float-only layout
    type: Horizontal
    children:
      - type: Leaf
      - type: Leaf
```

If `structure` is omitted or `null`, no windows are tiled under this layout — everything floats.

---

## 2. `TwmPluginNode` — Node Fields

Each node in the tree (including the root) supports:

| Field           | Type                                            | Default     | Meaning                                                                    |
| --------------- | ----------------------------------------------- | ----------- | -------------------------------------------------------------------------- |
| `type` / `kind` | `Leaf` \| `Stack` \| `Vertical` \| `Horizontal` | —           | Node kind. `type` and `kind` are interchangeable keys in YAML.             |
| `lifetime`      | `Permanent` \| `Temporal`                       | `Permanent` | Whether the node persists once emptied or is cleaned up.                   |
| `priority`      | number                                          | `1`         | Traversal order among sibling nodes — lower is tried first.                |
| `growFactor`    | number                                          | `1.0`       | Relative share of the parent's remaining space.                            |
| `condition`     | `TwmCondition` or omitted                       | none        | Gates whether this node currently accepts new windows. See §4.             |
| `children`      | `TwmPluginNode[]`                               | `[]`        | Only meaningful for `Vertical` / `Horizontal`; ignored for `Leaf`/`Stack`. |
| `maxStackSize`  | number or omitted                               | `3`         | Only meaningful for `Stack`; omit/`null` for unlimited.                    |
| `stackPolicy`   | `Manual` \| `AutoWhenOverflow` \| `Auto`        | `Auto`      | Only meaningful for `Stack` — see §3.                                      |

### Node kinds

| Kind         | Role                                                                                    |
| ------------ | --------------------------------------------------------------------------------------- |
| `Leaf`       | A single window slot. Accepts a window only while empty.                                |
| `Stack`      | Multiple windows tabbed together in one slot. Governed by `stackPolicy`/`maxStackSize`. |
| `Vertical`   | Container — splits `children` top/bottom. Never holds a window directly.                |
| `Horizontal` | Container — splits `children` left/right. Never holds a window directly.                |

### `stackPolicy` (Stack nodes only)

| Value              | Behavior                                                              |
| ------------------ | --------------------------------------------------------------------- |
| `Auto` (default)   | Freely accepts windows up to `maxStackSize`.                          |
| `AutoWhenOverflow` | Only accepts windows once other nodes can no longer take them.        |
| `Manual`           | Never accepts a window automatically — only via explicit user action. |

---

## 3. `condition` — `TwmCondition`

A boolean expression evaluated against the current tiling state, gating whether a node currently accepts windows.

```yaml
condition:
  compare:
    left: tiling-windows # or: is-reindexing
    op: ge # eq | ne | lt | le | gt | ge
    right: 4
```

Compound conditions:

```yaml
condition:
  or:
    - compare: { left: tiling-windows, op: eq, right: 3 }
    - compare: { left: tiling-windows, op: ge, right: 5 }
```

```yaml
condition:
  and:
    - compare: { left: tiling-windows, op: ge, right: 2 }
    - compare: { left: tiling-windows, op: lt, right: 8 }
```

```yaml
condition:
  not:
    compare: { left: is-reindexing, op: eq, right: true }
```

`left` operands:

| Operand          | Value                                                               |
| ---------------- | ------------------------------------------------------------------- |
| `tiling-windows` | Current count of tiled windows in the workspace.                    |
| `is-reindexing`  | Whether the WM is mid-reindex (bulk window reassignment) right now. |

`op` comparators: `eq`, `ne`, `lt`, `le`, `gt`, `ge` — standard numeric/equality comparison against `right` (a plain
YAML/JSON value: number, string, or boolean depending on the operand).

---

## 4. How It Becomes a Live Layout

At runtime, Seelen UI resolves the active layout's `Plugin` resource, parses its `structure` into `TwmPluginNode`s, and
converts that declarative tree into a runtime tree (`TwmRuntimeNode`) — copying `kind`, `lifetime`, `priority`,
`growFactor` (used as both the initial and the live, user-resizable grow factor), `condition`, `maxStackSize`, and
`stackPolicy` as-is, and initializing runtime-only state (assigned windows, active window, screen rect) empty. From then
on, whenever a window needs a slot, the WM walks the tree in `priority` order and asks each node `accepts_windows?` —
which checks the node's `condition` (if any) against the current `{ tiling-windows,
is-reindexing }` context, and then
applies kind-specific rules (`Leaf` only if empty; `Stack` only per its `stackPolicy`/`maxStackSize`; containers never
accept directly, only their children do).

Monocle mode (`Win + M`) is not a separate mechanism — it's implemented as the exact same schema, swapped in as
`{ kind: Stack, maxStackSize: null }` (unlimited stack, one slot).

---

## 5. Full Examples (Bundled Layouts)

**`@default/wm-columns`** — flattest possible layout, three fixed columns:

```yaml
id: "@default/wm-columns"
target: "@seelen/window-manager"
plugin:
  structure:
    type: Horizontal
    children:
      - type: Leaf
      - type: Leaf
      - type: Leaf
```

**`@default/wm-bspwm`** — recursive binary space partition:

```yaml
id: "@default/wm-bspwm"
target: "@seelen/window-manager"
plugin:
  structure:
    type: Horizontal
    children:
      - type: Leaf # 1st window
      - type: Vertical
        children:
          - type: Leaf # 2nd window
          - type: Horizontal
            children:
              - type: Vertical
                priority: 2
                children:
                  - type: Horizontal
                    priority: 2
                    children:
                      - type: Leaf # 5th window
                      - type: Leaf # 6th window
                  - type: Leaf # 4th window
                    priority: 1
              - type: Leaf # 3rd window
                priority: 1
```

**`@default/wm-grid`** — a dynamic grid using `condition` to reshape as window count changes:

```yaml
id: "@default/wm-grid"
target: "@seelen/window-manager"
plugin:
  structure:
    type: Horizontal
    children:
      - type: Vertical
        priority: 3
        condition:
          compare: { left: tiling-windows, op: ge, right: 4 }
        children:
          - type: Leaf
            priority: 3
            condition:
              compare: { left: tiling-windows, op: ge, right: 7 }
          - type: Leaf
          - type: Leaf
      - type: Vertical
        priority: 1
        children:
          - type: Leaf
            priority: 3
            condition:
              compare: { left: tiling-windows, op: eq, right: 8 }
          - type: Leaf
            priority: 1
          - type: Leaf
            priority: 2
            condition:
              or:
                - compare: { left: tiling-windows, op: eq, right: 3 }
                - compare: { left: tiling-windows, op: ge, right: 5 }
      - type: Vertical
        priority: 2
        children:
          - type: Leaf
            priority: 3
            condition:
              compare: { left: tiling-windows, op: ge, right: 6 }
          - type: Leaf
          - type: Leaf
```
