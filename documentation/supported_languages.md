# Supported Languages

Seelen UI maintains a single, authoritative list of the languages it supports across the app: the Settings language
picker, `i18n/translations/` locale files, and the
[`slu resource translate`](resource_text.md#4-translating-with-the-slu-cli) command all draw from this same list.

## Where it's defined

The list lives in Rust at:

```
libs/core/src/constants/mod.rs
```

as the `SUPPORTED_LANGUAGES` constant, an array of `SupportedLanguage { label, en_label, value }` entries — `label` is
the language's native name, `en_label` is its English name, and `value` is the language code (e.g. `es`, `pt-BR`,
`zh-CN`).

`libs/core` generates TypeScript bindings for the frontend from this same source, exported as `SupportedLanguages` in:

```
libs/core/src/constants/mod.ts
```

**Never hand-edit `mod.ts`** — it is generated. To add, remove, or rename a supported language, edit
`SUPPORTED_LANGUAGES` in `mod.rs` and regenerate bindings with `cd libs/core && deno task build:rs`.

## Why this matters for resources

Anything that consumes "which languages does Seelen UI support" — the Settings language dropdown, and the
`slu resource translate` command described in [resource_text.md](resource_text.md) — reads from this list. Adding a
language here automatically:

- Makes it selectable in the Settings language picker (once a matching locale file exists under `i18n/translations/`).
- Makes `slu resource translate` fill it in for every `ResourceText` field in a resource, with no changes needed on the
  resource author's side.

Because both frontend and CLI derive from the same Rust source, the list never drifts between "languages the UI offers"
and "languages resources get translated into" — check `mod.rs` directly for the current, exact list rather than keeping
a separate copy of it in documentation.
