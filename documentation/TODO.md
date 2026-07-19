# Documentation TODO

Gaps between what `documentation/` currently covers and what a developer or client actually needs to build a widget,
theme, plugin, icon pack, or wallpaper — or to integrate with Seelen UI's data/IPC layer. Grouped by priority. Each item
lists a suggested filename and category so it can be dropped straight into `docs-manifest.json` once written.

When a doc below is written: add its entry to `docs-manifest.json` under the listed category and flip planning here to
done (or just delete the line).

---

## P0 — Blocking for third-party widget/plugin development

These are referenced constantly by existing docs ("read the widget's documentation", "call the SLU library") but the
referenced thing doesn't exist yet.

- [x] ~~`widget_js_api.md`~~ — done as [`widget-js-api`](./widget-js-api) (category: `widgets`): covers
      `Widget.self.init()`/`.ready()`, the `invoke`/`subscribe` wrappers, and points to
      `libs/core/src/handlers/commands.rs`/`events.rs` as the source of truth instead of listing every command/event by
      hand. `LazySignal`-from-a-widget's-perspective is still not covered — worth a follow-up section if it comes up in
      practice.
- [x] ~~`toolbar_item_scripting.md`~~ — done as [`toolbar-plugins`](./toolbar-plugins) (category: `plugins`).
- [x] ~~`window_manager_layout_plugin.md`~~ — done as [`wm-layouts`](./wm-layouts) (category: `plugins`).
- [x] ~~dock plugin schema~~ — not originally tracked here, but the same gap existed for `@seelen/weg`; done as
      [`dock-plugins`](./dock-plugins) (category: `plugins`).

## P1 — Needed to actually publish/distribute a resource

- [ ] **`icon_pack_guidelines.md`** (category: `resource-kinds`) — `IconPack` is listed as a resource kind in
      resource_guidelines.md but has no dedicated guide: entry matching by path/UMID/process name, match priority when
      multiple packs are active, the `missing` fallback icon, remote entry format and caching.
- [ ] **`wallpaper_guidelines.md`** (category: `resource-kinds`) — `Wallpaper` resource kind has no dedicated guide:
      metadata for each `Wallpaper Type` (`Image`, `Video`, `Layered`, `MediaPlayer`), and specifically the `Layered`
      HTML/CSS/JS schema and the `MediaPlayer` sync contract — these are custom-code wallpaper types with no documented
      authoring surface at all.
- [ ] **`publishing_and_marketplace.md`** (category: `resources`) — End-to-end publish workflow: creating a Seelen
      account, uploading the bundled `.yaml` via the website, image requirements already noted in resource_guidelines.md
      (portrait/banner/screenshots dimensions) but not the review/approval process, how `appTargetVersion` compatibility
      is enforced, and how to push an update to an already-published resource (versioning behavior, whether old installs
      auto-update).
- [ ] **`slu_cli_reference.md`** (category: `resources`) — A single authoritative command reference. Today `load`/
      `unload`/`bundle`/`translate` are each documented inline in resource_guidelines.md and resource_text.md, but
      there's no page listing every `slu` subcommand (e.g. validation/lint commands, `slu doctor`-style diagnostics if
      they exist) in one place — needed before a docs-viewer nav item like "CLI" makes sense.

## P2 — Quality-of-life for resource authors

- [ ] **`widget_shortcuts.md`** or a section added to `widget-guidelines` — widgets can declare their own shortcuts, but
      `widget-guidelines`'s `metadata.yml` reference never shows the field for declaring a widget-owned shortcut, its
      default keys, or how the widget receives the trigger at runtime.
- [ ] **`popup_widget_api.md`** (category: `reference`) — Seelen UI ships a built-in Popup Widget (typed content blocks:
      `text`/`icon`/`image`/`button`/`group`; title/content/footer zones) usable by third-party widgets and plugins, but
      there's no doc for how to actually invoke it (which command, what payload shape, positioning options relative to a
      trigger element).
- [ ] **`debugging_resources.md`** (category: `resources`) — Common validation failures (e.g. missing `en` key, bad
      resource ID format) and where to look: log folder location, DevTools per-widget (already covered in widget/theme
      guidelines individually — this page would just consolidate the "my resource won't load, now what" troubleshooting
      flow in one place instead of three).

## P3 — Nice to have, lower urgency

- [ ] **`app_rules_authoring.md`** — App Rules are a fully-featured Settings UI concept (match by exe/class/title/path,
      matching strategies, extra flags like `WmFloat`/`VdPinned`); a short doc showing the raw YAML/JSON shape of an
      `AppIdentifier` + rule (for anyone scripting rules outside the Settings UI) would help power users.
- [ ] **`contributor_architecture.md`** — AGENTS.md already covers the modern backend module pattern for core
      contributors; if the docs viewer is meant to also serve contributors (not just resource authors), a short "start
      here" page linking AGENTS.md's conventions would avoid duplicating it here.

---

## Not gaps (confirmed intentionally undocumented)

- **Sound Packs** — feature is a placeholder, not implemented. No resource kind exists for it yet
  (`resource_guidelines.md` doesn't list it), so no doc is missing here — revisit if/when it ships.
