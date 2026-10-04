# Seelen UI — Design System & Manifesto

This document defines **how Seelen UI looks and why**.

**Scope:**

- **Built-in widgets and the default theme: mandatory.** It is a contract, not a suggestion. Any developer or AI agent
  changing built-in UI code, styles, or the default theme must follow it. If a change conflicts with this document, the
  change is wrong — not the document. Changing the design language itself requires an explicit decision from the
  maintainers and an update to this file in the same commit.
- **Third-party widgets and themes: guide.** Not enforced, but following it is how a community widget integrates
  correctly with the rest of the UI and with user themes that apply global changes (recolors, radius, spacing). A widget
  that hardcodes its look will not follow those themes.

---

## Table of Contents

1. [Manifesto](#1-manifesto)
2. [Where Styles Live](#2-where-styles-live)
3. [Design Tokens](#3-design-tokens)
4. [Light / Dark Mode](#4-light--dark-mode)
5. [Standard Components (`data-skin`)](#5-standard-components-data-skin)
6. [Writing Widget Markup](#6-writing-widget-markup)
7. [Writing Theme Styles](#7-writing-theme-styles)
8. [Forbidden Patterns](#8-forbidden-patterns)
9. [Checklist](#9-checklist)

---

## 1. Manifesto

**Simple as possible, everywhere.** Every rule below derives from these principles:

0. **UX comes first.** Every rule below serves usability. When a rule would hurt usability (illegible text, a state the
   user cannot perceive, a hidden affordance), the usability need wins. Such exceptions must be **functional**, never
   aesthetic, and must stay as small as possible. See [UX exceptions](#ux-exceptions).
1. **Flat design.** Solid, opaque surfaces, subtle shadows (`--shadow-s/m/l`), soft rounded corners. No gradients,
   glows, glassmorphism, blur, skeuomorphism, borders, or "eye candy" in the default look. Depth is communicated by
   surface tone and shadow — nothing else.
2. **Dynamic by default.** Every color comes from a variable. UI colors adapt to the user's **light/dark color scheme**
   and **Windows accent color**, so the UI looks correct in both schemes without any extra code.
3. **Standard components, defined once.** Buttons, inputs, selects, sliders, switches, checkboxes, radios and popovers
   are defined once in the default theme's shared styles and reused everywhere through `data-skin`. Do not create
   one-off variants.
4. **Moddable first.** Users restyle Seelen UI through themes. The simpler and flatter our markup and CSS are, the
   easier it is to theme. Low specificity, descriptive class names, no inline visual styles, no visuals baked into
   components.
5. **Consistency over novelty.** A new widget must look like it has always been part of Seelen UI. Reuse existing
   tokens, skins and patterns instead of inventing new ones.

> **This document wins over existing code.** Parts of the codebase still contain legacy patterns that are pending
> migration (hardcoded spacing/radius, borders, other font weights…; the main ones are marked with "Existing code
> still…" notes below). Never copy a pattern from existing code that this document forbids.

> If a design idea needs more than the existing tokens and skins, simplify the idea. Extending the system (new tokens,
> skins or exceptions) requires a maintainer decision and an update to this document.

### UX exceptions

An exception to the rules in this document is valid only when it solves a concrete usability problem: **legibility**,
**state feedback** (something is loading, something needs attention), or **affordance** (something can be scrolled,
something is modal). "It looks better" is not a valid reason, except for the explicit design exception of translucent
desktop bars below.

This table is the complete list of accepted exceptions. Use it as precedent: a new exception must meet the same bar and
is only valid once a maintainer approves it and it is added here.

| Exception                                                       | Where                                                                                   | Why                                                                              |
| --------------------------------------------------------------- | --------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| Semi-transparent background (derived from `--slu-std-bg-color`) | Toolbar, dock (including its window preview) and window manager stack bar **only**      | Design exception: always-visible desktop bars blend with the wallpaper. No blur. |
| Dark translucent full-screen overlay (scrim)                    | Behind full-screen modals (e.g. power menu)                                             | Signals a modal state covering the whole desktop. The modal itself is standard.  |
| `text-shadow`                                                   | Text drawn over dynamic images (e.g. media album art)                                   | Legibility over unpredictable backgrounds.                                       |
| Inset edge shadows                                              | Scrollable containers with overflow (e.g. dock)                                         | Affordance: tells the user there is more content to scroll.                      |
| Infinite animation                                              | Loading spinners, "needs attention" indicators                                          | State feedback: the state lasts until it changes.                                |
| Focus ring (`outline`)                                          | Keyboard focus (`:focus-visible`)                                                       | Accessibility: shows where keyboard focus is.                                    |
| Accent `outline`                                                | Active window (window manager), current selection in task switcher, drag & drop targets | State feedback / affordance: the line _is_ the indicator.                        |
| Control `border`                                                | Unchecked checkbox / radio                                                              | Affordance: without it the control is invisible.                                 |

Exceptions change _what_ is drawn, never _how colors are written_: their colors still come from variables
(`--shadow-color` for shadows, `--color-fixed-*` for scrims), e.g. `hsl(var(--shadow-color) / 0.5)`.

Translucency is **never** allowed anywhere else: popovers, menus, dialogs, cards and every other widget surface are
opaque.

---

## 2. Where Styles Live

| Layer                      | Location                                                        | Contains                                                                                                         |
| -------------------------- | --------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| Design tokens              | `libs/core/styles/` (`colors`, `spacings`, `shadows`, `radius`) | Color scales, spacing scale, shadows, radius scale. Injected into every webview by `src/ui/vanilla/entry-point`. |
| Reset                      | `libs/core/styles/reset.css` (`@layer reset`)                   | Neutral reset, `color-scheme: light dark`. No visuals.                                                           |
| Shared standard components | `src/static/themes/default/shared/`                             | `--slu-std-*` semantic tokens, `data-skin` components, `.slu-std-popover`, scrollbars, focus ring.               |
| Widget visuals             | `src/static/themes/default/styles/<widget>.scss`                | The look of each built-in widget.                                                                                |
| Widget component CSS       | `<style>` in `.svelte` / CSS Modules in React                   | **Functional/structural only** (layout mechanics, visibility states). Never colors or decoration.                |

Key consequence: **the visual identity of built-in widgets lives in the default theme**, not in the components. This is
what lets a user theme replace it completely. The default theme is loaded through CSS cascade layers
(`@layer theme-<id>-shared` and `@layer theme-<id>`, see `libs/core/src/state/theme/mod.ts`), so user themes stacked on
top win without `!important` battles.

---

## 3. Design Tokens

Always use tokens. **Colors are never hardcoded** (no hex, `rgb()`, `hsl()` literals or named colors like `white`), not
even for exceptions — always a standard variable. Shadows, spacing (padding, margin, gap) and radius always use their
tokens too. Element dimensions (width, height) are not tokenized.

When a color must stay the same in both schemes (e.g. the light background a QR code needs to be scannable, a fallback
behind the wallpaper), use `--color-fixed-*`.

### Colors — contrast-adaptive scales (`libs/core/styles/colors.css`)

`--color-<hue>-<step>` where the step is **contrast against the current background**, not absolute lightness:

- `25–100` = low contrast (surfaces, subtle fills), `900` = maximum contrast (text).
- Scales invert automatically between light and dark, so `--color-gray-900` is always "text-like" and `--color-gray-50`
  is always "surface-like".
- Hues: `gray`, `red`, `orange`, `yellow`, `green`, `seafoam`, `cyan`, `blue`, `indigo`, `purple`, `fuchsia`, `magenta`.
- **`gray` is the neutral scale** for surfaces, text and scrollbars.
- **Every other hue is reserved for meaning**, i.e. places where the color itself tells the user something: errors
  (red), warnings (yellow), success (green), info (blue), and similar generic indicators (badges, status dots,
  battery/level alerts, category tags). They are **never** used as accent, decoration, or to "brand" a widget.
- `--color-fixed-*` never changes with the scheme. Only for content that must keep its color in both schemes (QR codes,
  fallbacks behind images, illustrations) — never for regular UI surfaces or text.

### Accent — `--system-accent-*`

Comes from the user's Windows accent color: `--system-accent-color` plus `light`, `lighter`, `lightest`, `dark`,
`darker`, `darkest` variants.

**Accent colors ALWAYS respect the accent defined by the user in Windows.** Anything that is "accented" (selected,
active, checked, primary action, focus ring, progress, highlights) uses the accent. Never substitute a hue from the
color scales (e.g. `--color-blue-*`) or a fixed brand color for the accent.

Which accent variable to use: the `--slu-std-ui-*` tokens (they already pick the right variant for light/dark). Use a
raw `--system-accent-*` variable only when no `--slu-std-ui-*` token covers the case (e.g. the focus ring and state
outlines).

### Semantic tokens — `--slu-std-*` (use these first)

Defined in `src/static/themes/default/shared/index.scss`. Widgets and theme styles use these first. Use a scale variable
(`--color-*`) directly only when no `--slu-std-*` token expresses the need (e.g. status hues, `--color-fixed-*`).

| Token                           | Use                                                                   |
| ------------------------------- | --------------------------------------------------------------------- |
| `--slu-std-fg-color`            | Primary text                                                          |
| `--slu-std-fg-secondary-color`  | Secondary text                                                        |
| `--slu-std-fg-muted-color`      | Hints, captions, icons of low importance                              |
| `--slu-std-fg-disabled-color`   | Disabled text                                                         |
| `--slu-std-bg-color`            | Default surface (body, popovers)                                      |
| `--slu-std-bg-light-color`      | Raised surface (cards, inputs, dialogs)                               |
| `--slu-std-bg-dark-color`       | Sunken surface                                                        |
| `--slu-std-ui-color`            | Accent fill: solid buttons, checked controls, single highlighted item |
| `--slu-std-ui-fg-color`         | Text/icons on top of `--slu-std-ui-color` (see below)                 |
| `--slu-std-ui-hover-color`      | Hover state of elements filled with `--slu-std-ui-color`              |
| `--slu-std-ui-hover-overlay`    | Hover background (transparent controls, list items, cells)            |
| `--slu-std-ui-pressed-overlay`  | Pressed background (transparent controls, list items, cells)          |
| `--slu-std-ui-selected-overlay` | Selected item in a list, or a tinted resting state                    |

**Elevation:** the `bg` tokens are elevation levels. `--slu-std-bg-color` is the base level (the widget/popover itself),
`--slu-std-bg-light-color` is one level up (cards, inputs, items placed on the base), and `--slu-std-bg-dark-color` is
one level down (wells, recessed areas). Pick the level by where the element sits relative to its parent, not by how it
looks; this is how flat design shows depth without borders.

**Text on accent:** `--slu-std-ui-color` is a dark accent in light scheme and a light accent in dark scheme, so the
foreground on top of it is the inverse of the normal foreground. Always use `--slu-std-ui-fg-color` for text/icons on
accent backgrounds (solid buttons, selected items, checked controls); never `white`/`black` or a guessed gray.

> Existing code still uses `--color-gray-100` directly for this; it will be migrated to `--slu-std-ui-fg-color` later.
> New code must use the variable.

**De-emphasis uses foreground tokens, not `opacity`.** Secondary/muted/disabled text and icons get
`--slu-std-fg-secondary-color`, `--slu-std-fg-muted-color` or `--slu-std-fg-disabled-color`. Lowering `opacity` on text
mixes it with whatever is behind it and breaks recolor themes. The only allowed `opacity` de-emphasis is the disabled
state (`opacity: 0.5`). `opacity` for show/hide transitions is fine.

**No free alphas.** Never derive ad-hoc transparent variants of a color (`oklch(from var(--x) l c h / 0.2)`,
`rgb(from …)`, `color-mix(…)`) in widget or theme styles. Accent tints come only from the overlay variables:

- `--slu-std-ui-hover-overlay` — hover.
- `--slu-std-ui-pressed-overlay` — pressed.
- `--slu-std-ui-selected-overlay` — selected item in a list, or a tinted resting state.

The only places where an alpha derivation is written by hand are the definitions of these variables in
`shared/index.scss` and the documented [UX exceptions](#ux-exceptions) (toolbar/dock/stack bar background, scrim,
shadows).

**Interactive states** (lists, cards, cells, menu items — anything that is not a shared `data-skin` control):

| State                               | Style                                                               |
| ----------------------------------- | ------------------------------------------------------------------- |
| Hover                               | `--slu-std-ui-hover-overlay`                                        |
| Pressed                             | `--slu-std-ui-pressed-overlay`                                      |
| Selected / active item in a list    | `--slu-std-ui-selected-overlay`                                     |
| Single highlighted item (e.g. day)  | `--slu-std-ui-color` background + `--slu-std-ui-fg-color` text      |
| Keyboard focus / keyboard selection | Focus ring `outline` (see [Borders & outlines](#borders--outlines)) |
| Disabled                            | `opacity: 0.5`                                                      |

Sizes of interactive items (and therefore click targets) are not fixed by this document: they come from the theme tokens
and, in some widgets (e.g. toolbar, dock), from the user's own settings.

**Status hues:** error = `red`, warning = `yellow`, success = `green`, info = `blue`. No other hue is used for these
states. There is no fixed step per status: pick the step that gives enough contrast where it is used (the scales already
adapt to the scheme). Use them only to convey that meaning (see above).

### Spacing — `--spacing-*` (`libs/core/styles/spacings.css`)

`2xs` 4px · `xs` 8px · `s` 12px · `m` 16px · `l` 20px · `xl` 24px · `2xl` 32px. Every padding, margin and gap uses one
of these.

### Shadows — `--shadow-*` (`libs/core/styles/shadows.css`)

`--shadow-s` (controls), `--shadow-m` (popovers, menus), `--shadow-l` (modal dialogs). No other shadows, except the ones
listed in [UX exceptions](#ux-exceptions).

### Radius — `--radius-*` (`libs/core/styles/radius.css`)

Radii are a small scale **by role**, not a value per component:

| Token           | Value    | Role                                              |
| --------------- | -------- | ------------------------------------------------- |
| `--radius-s`    | `6px`    | Controls: buttons, inputs, selects, list items    |
| `--radius-m`    | `10px`   | Surfaces: popovers, menus, cards                  |
| `--radius-l`    | `16px`   | Large containers: dialogs, full-screen panels     |
| `--radius-full` | `9999px` | Pills: switches, tags, chips                      |
| `50%`           | —        | Circles only: avatars, status dots, round buttons |

Rules:

- **Corners are concentric.** When an element sits inside a rounded container, its radius is the container's radius
  minus the padding between them, so both curves share the same center:

  ```scss
  .card {
    border-radius: var(--radius-m);
    padding: var(--spacing-xs);

    > .card-item {
      border-radius: max(0px, calc(var(--radius-m) - var(--spacing-xs)));
    }
  }
  ```

  If the result is 0 or less, the inner element is square (that is correct). Never replace the formula with a different
  number.
- No percentages other than `50%` (percent radii are computed per axis and produce elliptical, size-dependent corners).
- No hardcoded radius values. A radius is always one of: a `--radius-*` token, a `calc()` derived from one, or a value
  derived from a user setting variable (e.g. the item size of the toolbar/dock). This keeps the whole UI re-shapeable by
  a theme changing just these variables.

> Existing code still has hardcoded radii; it will be migrated to these tokens later. New code must use the tokens.

### Borders & outlines

**`border` — never.** Flat design does not use borders: no framed boxes, no framed inputs, no 1px divider lines.
Instead:

- **Group / separate content** with spacing (`--spacing-*`) and surface tone (`--slu-std-bg-*`).
- **Show interactivity** with a surface fill (`--slu-std-bg-light-color`) and the hover/pressed overlays.

The only `border` allowed is the one that draws an unchecked checkbox/radio (without it the control is invisible).

**`outline` — reserved for focus and state.** `outline` is the UX tool to point at something, never decoration:

- Keyboard focus ring (`:focus-visible`, already defined in shared styles).
- State indicators where the line _is_ the information: active window, current selection (e.g. task switcher), drag &
  drop targets, error on a field.

Always with accent or status color variables; never to frame a box or separate content.

> Existing code still has borders and dividers (and some state indicators drawn with `border`); they will be migrated
> later. New code must follow these rules.

### Typography

- Font: **always the inherited system font**. No other font families, including `monospace` for technical data (IPs,
  MACs, codes): the UI never shows content that needs vertical alignment of characters.
- Sizes: Seelen UI is a desktop UI, not a document, so there is no typographic scale. Only two sizes:
  - `1rem` — the base. It is the user's browser/system default, so never override the root font size.
  - `0.8rem` — menus and popovers (`.slu-std-popover` already sets it).

  Hierarchy (titles vs. body vs. captions) is expressed with font weight and the `--slu-std-fg-*` tokens, not with new
  sizes. Another size is allowed only when the content itself needs it (e.g. a pairing code meant to be read from a
  distance), never to build a hierarchy.
- Font size units: always `rem`/`em`, never `px`, so text follows the user's settings.
- Weights: regular (`400`), `500`, `600`. No other weights.

### Icons

- Rendered through the shared `Icon` components in `libs/ui/svelte/components/Icon`; they inherit `currentColor`.
- **Default size = the font size** (`1em`), so an icon next to text always matches it.
- Icons are images, so a bigger icon is allowed when the UI needs it (e.g. quick settings tiles, avatars, app icons).
  Size it in `em`/`rem`, or from a user setting variable when the widget has one (e.g. dock item size).

### Motion

**Avoid animations by default.** Only animate where motion is needed, such as the standard widget show/hide (already
defined in shared styles), state changes of controls, and infinite animations that communicate an ongoing state
(spinners, attention indicators). Keep them short and simple.

- Forbidden: decorative motion — animated backgrounds, ambient/idle effects, complex or showy hover animations.
- The app itself stops/skips **all** animations when the user enables extreme performance mode or the OS asks for
  reduced motion (`prefers-reduced-motion`). Themes and widgets do not need to handle either case, but must never depend
  on an animation for correctness (a loading state must still be recognizable without the spin).

---

## 4. Light / Dark Mode

- `color-scheme: light dark` is set at the root; the scheme follows the OS (`prefers-color-scheme`).
- Using `--color-*` and `--slu-std-*` tokens gives you both modes **for free**. This is the expected path.
- When a value really must differ between modes, use CSS `light-dark(<light>, <dark>)` with tokens inside, as
  `shared/index.scss` does. Do not write separate `@media (prefers-color-scheme)` blocks in widget/theme styles.
- In JS, only read the scheme (`libs/ui/svelte/runes/DarkMode.svelte.ts`) for non-CSS needs (e.g. picking an image).
  Never compute colors in JS.
- Every UI change must be checked in **both** light and dark mode, and with a non-default accent color.

---

## 5. Standard Components (`data-skin`)

Shared components are opt-in through the `data-skin` attribute. Shared styles never target bare elements (`button`,
`input`, `select`) — only `[data-skin=...]` — so widgets and third-party themes never get surprise styles.

| Element                             | Skins                                    |
| ----------------------------------- | ---------------------------------------- |
| `button`                            | `default`, `solid`, `transparent`        |
| `input` (text-like types), `select` | `default`, `transparent`                 |
| `input[type="checkbox"]`            | `default`, `switch`                      |
| `input[type="radio"]`               | `default`                                |
| `input[type="range"]`               | `flat` (+ `data-orientation="vertical"`) |
| `div[data-behavior="group"]`        | Joins adjacent buttons into one group    |
| `.slu-std-popover`                  | Standard popup surface                   |

Meaning of each button skin:

- `solid` — the **one** primary action of a view (accent background).
- `default` — secondary actions (raised surface).
- `transparent` — toolbar-like / icon actions (only a hover overlay).

Other shared state attributes: `data-error="true"` on inputs, `:disabled` (rendered at 50% opacity).

```svelte
<div class="slu-std-popover">
  <input type="search" data-skin="default" placeholder={t("search")} />
  <div data-behavior="group">
    <button data-skin="default" onclick={onCancel}>{t("cancel")}</button>
    <button data-skin="solid" onclick={onAccept}>{t("accept")}</button>
  </div>
</div>
```

**Need a new skin or component?**

- Used by more than one widget → add it to `src/static/themes/default/shared/`, built only from tokens, and document it
  in this table.
- Used by one widget only → style it in that widget's theme file, built only from tokens.
- Never fork or re-style an existing skin inside a widget.

---

## 6. Writing Widget Markup

- Use **intuitive, descriptive class names** in kebab-case that say what the element is or does (`.power-menu-user`,
  `.calendar-cell-today`). There is no mandatory prefix scheme.
- Themes target these classes, so **avoid renaming or removing them without a reason**. It is not forbidden: themes
  declare a target version and their authors are responsible for updating them after app changes.
- Keep the DOM flat and predictable. No wrapper elements that exist only for styling.
- Use `data-*` attributes for state (`data-active`, `data-showing`, `data-orientation`) so themes can style states
  without JS.
- Component-scoped CSS (`<style>` in Svelte, CSS Modules in React) is **only** for functional behavior: layout
  mechanics, visibility toggles, overflow, sizing that the component needs to work. Colors, backgrounds, borders,
  radius, shadows and decoration go to the theme file `src/static/themes/default/styles/<widget>.scss`.
- No inline `style=` for visuals. Inline styles are only acceptable for runtime-computed geometry (position, size,
  scale), and preferably exposed as CSS variables (`style="--progress: {value}"`) so themes can still use them.
- All visible text goes through i18n.

---

## 7. Writing Theme Styles

Mandatory for the default theme; the recommended practice for community themes:

- Target the widget's root class and nest from there, only as deep as the structure requires. No `!important`, no ID
  selectors.
- Use only tokens (`--slu-std-*`, `--color-*`, `--system-accent-*`, `--spacing-*`, `--shadow-*`, `--radius-*`).
- Reuse standard components instead of re-styling raw controls.
- Do not restyle global bare elements in `sharedStyles` (except `body` base colors, already defined).

See [theme guidelines](./theme-guidelines) for the theme resource format itself.

---

## 8. Forbidden Patterns

Reject these in review, regardless of who (person or AI) wrote them:

- Hardcoded colors (`#fff`, `rgb(0 0 0 / .5)`, `black`, `white`), anywhere, for any reason. Use a variable.
- Font families other than the inherited system font (including `monospace`).
- Gradients, glows, blur/acrylic backdrops, neon effects.
- Shadows other than the `--shadow-*` tokens, outside the [UX exceptions](#ux-exceptions).
- `border` or divider lines to draw boxes, frame inputs or separate content; `outline` used for anything other than
  focus or state indication (see [Borders & outlines](#borders--outlines)).
- Free alpha variants of colors (`oklch(from var(--x) l c h / 0.2)`, `rgb(from …)`, etc.) instead of the standard
  overlay variables.
- Translucent surfaces outside the toolbar / dock / window manager stack bar exception.
- Decorative animations (animated backgrounds, idle effects, showy hover animations).
- "Exceptions" justified by looks instead of a functional UX need (see [UX exceptions](#ux-exceptions)).
- Colors chosen in JS/TS, or `@media (prefers-color-scheme)` blocks instead of tokens/`light-dark()`.
- A fixed brand color or a scale hue (`--color-blue-*`, etc.) used where the user's accent belongs.
- Scale hues used for decoration instead of conveying a status/meaning to the user.
- New one-off button/input styles inside a widget instead of `data-skin`.
- Visual styles inside component-scoped CSS or inline `style=`.
- Shared styles targeting bare `button`, `input`, `select`, etc.
- Pulling in a UI kit or CSS framework (Tailwind, component libraries) into widgets.
- "Redesigns" or visual refreshes that were not explicitly requested by the maintainers.

> **Settings app** (`src/ui/react/settings`): a legacy, complex app built with Ant Design that does not load themes. It
> is not held to every rule here; its priority is UX. It should still follow the manifesto as closely as it reasonably
> can (tokens, flat, light/dark, accent), and its dependencies must not spread to widgets.

---

## 9. Checklist

Before submitting any UI change:

- [ ] Looks correct in light **and** dark mode, and with a different Windows accent color.
- [ ] Only tokens used; no hardcoded colors/shadows/spacing/radius.
- [ ] Text de-emphasis via `--slu-std-fg-*`, not `opacity`; text on accent uses `--slu-std-ui-fg-color`.
- [ ] Nested rounded elements have concentric corners.
- [ ] Controls use `data-skin`; popups use `.slu-std-popover`.
- [ ] Visual CSS lives in the default theme, not in the component.
- [ ] Class names are intuitive kebab-case (not renamed without reason); states exposed via `data-*`.
- [ ] No `border` or divider lines; `outline` only for focus/state indicators.
- [ ] Still flat and simple — nothing added "for style".
- [ ] Animations only where needed, and the UI works with all animations disabled.
- [ ] All text is i18n, rendered in the system font.
