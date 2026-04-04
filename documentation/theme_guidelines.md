# Seelen UI — Theme Guidelines

Reference guide for creating themes. A theme customizes the look of Seelen UI widgets by providing CSS (or SCSS) styles
and optional user-configurable CSS variables.

Read [resource_guidelines](./resource_guidelines) first for concepts shared across all resource kinds.

---

## Table of Contents

1. [Theme Structure](#1-theme-structure)
2. [Targeting Widgets](#2-targeting-widgets)
3. [Shared Styles](#3-shared-styles)
4. [User-Configurable Settings (CSS Variables)](#4-user-configurable-settings-css-variables)
   - [Color](#color)
   - [Number](#number)
   - [Length / Percentage](#length--percentage)
   - [String](#string)
   - [URL](#url)
   - [Grouping Settings](#grouping-settings)
5. [System CSS Variables](#5-system-css-variables)
6. [Animations and Performance Mode](#6-animations-and-performance-mode)
7. [Folder Structure](#7-folder-structure)
8. [Inspecting Widgets with DevTools](#8-inspecting-widgets-with-devtools)

---

## 1. Theme Structure

A theme is a folder with a `metadata.yml` at its root. The file has four main sections:

```yaml
# my-theme/metadata.yml

id: "@yourname/my-theme"

metadata:
  displayName: My Theme
  description: A short description.
  tags:
    - dark
    - minimal

# CSS for individual widgets (one entry per widget)
styles:
  "@seelen/fancy-toolbar": !include styles/toolbar.scss
  "@seelen/weg": !include styles/weg.css

# CSS applied to every widget (optional)
sharedStyles: !include shared/index.scss

# User-configurable CSS variables (optional)
settings:
  - syntax: <color>
    name: --my-accent
    label: Accent color
    initialValue: "#6c63ff"
```

All four sections are independent — you can have only `styles`, only `settings`, or any combination.

---

## 2. Targeting Widgets

The `styles` map uses the widget's ID as the key. Each entry is a CSS or SCSS string that is injected exclusively into
that widget's webview.

A widget ID follows the same format as any resource ID: `@creator/name` for local and community widgets, or a UUID for
widgets downloaded from the marketplace.

```yaml
styles:
  # Local / community widget (friendly ID)
  "@seelen/fancy-toolbar": !include styles/toolbar.scss

  # Another creator's widget
  "@someone/custom-clock": !include styles/custom-clock.scss

  # Downloaded widget identified by UUID
  "550e8400-e29b-41d4-a716-446655440000": !include styles/downloaded-widget.scss
```

You can find a widget's ID in the Seelen UI settings panel, on the widget's detail page. You don't have to target every
widget — include only the ones your theme actually reskins.

---

## 3. Shared Styles

`sharedStyles` is a single CSS/SCSS block that is injected into **every** widget. Use it for things that should look the
same everywhere — scrollbars, focus rings, popover containers, shared components.

```yaml
sharedStyles: !include shared/index.scss
```

**Important convention:** shared styles must not override global bare elements like `body`, `button`, or `input`
directly. Seelen UI widgets use **`data-skin` attributes** to opt into shared component styles. Only target those
attributes in shared styles:

```css
/* OK — targets only elements that explicitly opt in */
button[data-skin="solid"] {
  background: var(--slu-std-ui-color);
}
button[data-skin="default"] {
  border: 1px solid var(--slu-std-ui-color);
}
button[data-skin="transparent"] {
  background: transparent;
}

input[type="checkbox"][data-skin="switch"] {
  /* toggle switch */
}
input[type="range"][data-skin="flat"] {
  /* flat slider */
}
select[data-skin="default"] {
  /* styled select */
}

/* OK — targets a specific component class */
.slu-std-popover {
  background: var(--color-gray-50);
}

/* Avoid — overrides every button everywhere */
button {
  background: red;
}
```

The `sharedStyles` field is also the right place to define the performance-mode rule (see
[section 6](#6-animations-and-performance-mode)).

---

## 4. User-Configurable Settings (CSS Variables)

The `settings` list lets you expose CSS variables that users can tweak from the Seelen UI settings panel without
touching your files. Each entry defines one variable, the type of input to show, and a default value.

Every variable name must start with `--` and follow CSS custom property naming rules (`--my-variable`,
`--toolbar-height`, `--Color_1`, etc.).

---

### Color

Shows a color picker with an alpha slider. Value is stored as a hex string.

```yaml
- syntax: <color>
  name: --my-background
  label: Background color
  initialValue: "#1e1e2e"
```

With optional description and tooltip:

```yaml
- syntax: <color>
  name: --my-accent
  label: Accent color
  description: Used for highlights and interactive elements.
  tip: Supports transparency.
  initialValue: "#6c63ff"
```

Use in CSS:

```css
.toolbar {
  background-color: var(--my-background);
  color: var(--my-accent);
}
```

---

### Number

Shows a plain number input. Add `step` to turn it into a slider. Add `options` to turn it into a dropdown. `min` and
`max` constrain the input range.

```yaml
# Plain input
- syntax: <number>
  name: --toolbar-opacity
  label: Toolbar opacity
  initialValue: 90
  min: 0
  max: 100

# Slider (step enables the slider UI)
- syntax: <number>
  name: --border-radius
  label: Corner radius
  initialValue: 8
  min: 0
  max: 24
  step: 1

# Dropdown
- syntax: <number>
  name: --font-size
  label: Font size
  initialValue: 13
  options:
    - 11
    - 13
    - 15
    - 17
```

Use in CSS:

```css
.ft-bar {
  opacity: calc(var(--toolbar-opacity) / 100);
}
.ft-bar-item {
  border-radius: calc(var(--border-radius) * 1px);
}
```

---

### Length / Percentage

Shows a number input where the user can also choose the unit (px, %, em, rem, vw…). Use this when the user should be
free to pick the unit. If you need a fixed unit, use `<number>` and do the unit conversion in CSS.

```yaml
- syntax: <length>
  name: --item-size
  label: Item size
  initialValue: 40
  initialValueUnit: px
```

Use in CSS:

```css
.weg-item {
  width: var(--item-size);
  height: var(--item-size);
}
```

---

### String

Shows a text input. Use `min` and `max` to enforce length limits.

```yaml
- syntax: <string>
  name: --custom-font
  label: Font family
  initialValue: "Segoe UI"
  max: 80
```

Use in CSS:

```css
* {
  font-family:
    var(--custom-font), sans-serif;
}
```

---

### URL

Shows a file picker. The selected file path is stored as a CSS `url()` value. The `initialValue` field is ignored — the
variable starts empty until the user picks a file.

```yaml
- syntax: <url>
  name: --custom-background-image
  label: Background image
  initialValue: ""
```

Use in CSS:

```css
.ft-bar::before {
  content: "";
  background-image: var(--custom-background-image);
}
```

---

### Grouping Settings

Use `group` to organize settings into collapsible sections. Groups can be nested.

```yaml
settings:
  - syntax: <color>
    name: --toolbar-bg
    label: Toolbar background
    initialValue: "#1e1e2e"

  - group:
      header: Dock settings
      items:
        - syntax: <color>
          name: --dock-bg
          label: Dock background
          initialValue: "#313244"

        - syntax: <number>
          name: --dock-item-size
          label: Item size
          initialValue: 40
          min: 24
          max: 72
          step: 2

        - group:
            header: Dock border
            items:
              - syntax: <color>
                name: --dock-border-color
                label: Border color
                initialValue: "#45475a"
              - syntax: <number>
                name: --dock-border-width
                label: Border width
                initialValue: 1
                min: 0
                max: 4
                step: 1
```

---

## 5. System CSS Variables

Seelen UI makes a set of CSS variables available in every widget. You can use these in your stylesheets without
declaring them yourself.

**Windows accent colors** — taken from the user's Windows accent color setting:

```css
--system-accent-color          /* main accent */
--system-accent-light-color    /* light variant */
--system-accent-lighter-color  /* lighter variant */
--system-accent-lightest-color /* lightest variant */
--system-accent-dark-color     /* dark variant */
--system-accent-darker-color   /* darker variant */
--system-accent-darkest-color  /* darkest variant */
```

---

## 6. Animations and Performance Mode

When the user enables **extreme performance mode** in Seelen UI settings, all animations and transitions defined in
themes are automatically forced off by the app. You don't need to do anything special — this is handled for you.

---

## 7. Folder Structure

Keep all style paths explicit in `metadata.yml` using `!include`. The entrypoint is the single source of truth about
which widgets your theme targets.

```
my-theme/
├── metadata.yml
├── i18n/
│   ├── display_name.yml
│   └── description.yml
├── shared/
│   └── index.scss
└── styles/
    ├── toolbar.scss
    └── weg.css
```

```yaml
# metadata.yml
styles:
  "@seelen/fancy-toolbar": !include styles/toolbar.scss
  "@seelen/weg": !include styles/weg.css
sharedStyles: !include shared/index.scss
```

---

## 8. Inspecting Widgets with DevTools

Every Seelen UI widget is a webview, so you can inspect its HTML, CSS, and JavaScript exactly like a webpage. Click on
any widget to focus it, then press **Ctrl + Shift + I** to open the browser DevTools for that widget.

From DevTools you can:

- Inspect the DOM and live-edit CSS to prototype styles before writing them in your files.
- Use the **Elements** panel to find the class names and data attributes your selectors should target.
- Use the **Console** to test JavaScript expressions (useful when developing widgets or plugins).
- Use the **Performance** panel to profile rendering and catch expensive animations or layouts.

This works on any widget at any time — no special build or dev mode required.
