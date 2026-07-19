# ResourceText — User-Facing Text in Resources

`ResourceText` is the type Seelen UI uses everywhere a resource needs to show text to the user: display names,
descriptions, widget setting labels, tooltips, context menu entries, shortcut labels, theme config labels, etc. This
guide explains what it is, everywhere it shows up, and how to translate it with the `slu` CLI.

---

## Table of Contents

1. [What is a ResourceText?](#1-what-is-a-resourcetext)
2. [Where ResourceText is Used](#2-where-resourcetext-is-used)
3. [Writing ResourceText in YAML](#3-writing-resourcetext-in-yaml)
4. [Translating with the `slu` CLI](#4-translating-with-the-slu-cli)
5. [Tips and Gotchas](#5-tips-and-gotchas)

---

## 1. What is a ResourceText?

`ResourceText` is a field that accepts **either**:

- A plain string, which is treated as English (`en`) only:

  ```yaml
  label: My Setting
  ```

- A map of language code to translated string:

  ```yaml
  label:
    en: My Setting
    es: Mi Ajuste
    de: Meine Einstellung
  ```

At render time, Seelen UI looks up the text for the user's current language and falls back to `en` if that language is
missing. Because of this fallback, **`en` must always be present** whenever you use the map form — a resource missing
the `en` entry fails validation on publish.

There is no need to decide up front which form to use: start with a plain string while developing, then convert it to a
map once you (or the `slu` CLI) add translations. Both forms are accepted anywhere a `ResourceText` field appears.

---

## 2. Where ResourceText is Used

`ResourceText` is not limited to `metadata.displayName` / `metadata.description`. It is the standard type for **any**
user-visible text across all resource kinds:

| Resource kind | Fields using `ResourceText`                                                                           |
| ------------- | ----------------------------------------------------------------------------------------------------- |
| All resources | `metadata.displayName`, `metadata.description`                                                        |
| Widget        | Setting group `label` / `description`, setting `label` / `description` / `tip`, select-option `label` |
| Theme         | Config group `header`, config variable `label` / `description` / `tip`                                |
| Plugin        | Context menu item `label` (for `Submenu` and other entries)                                           |
| Shortcuts     | Shortcut `label` / `description`                                                                      |

In short: wherever you see a `label`, `description`, or `tip` field in a resource schema, assume it is a `ResourceText`
and can be localized the same way `displayName` is.

---

## 3. Writing ResourceText in YAML

For a short label, write it inline:

```yaml
label:
  en: Enable Feature
  es: Activar Función
```

For anything longer (a resource's `displayName`/`description`, or many settings with many languages), split each text
into its own file under `i18n/` and pull it in with `!extend`, exactly like described in the
[Extended YAML section](resource_guidelines.md#extend--embed-a-yaml-file) of the resource guidelines:

```yaml
# metadata.yml
metadata:
  displayName: !extend i18n/display_name.yml
  description: !extend i18n/description.yml
```

```yaml
# i18n/display_name.yml
en: My Dark Theme
es: Mi Tema Oscuro
```

This keeps `metadata.yml` readable and gives each translated field its own file that the `slu` CLI can translate
independently (see next section).

---

## 4. Translating with the `slu` CLI

Seelen UI ships a `slu resource translate` command that fills in every supported language for a `ResourceText` YAML
file, using **Google Translate** under the hood.

```bash
slu resource translate <path/to/file.yml> [source_lang]
```

- `<path/to/file.yml>` — a YAML file containing a `ResourceText` value (either a plain string or a language map).
- `[source_lang]` — the language code your source text is written in. Defaults to `en`.

### How it works

1. The command reads the file and looks for an entry matching `source_lang`. If it's missing, the command fails — write
   your source text first.
2. It iterates over every language in [Seelen UI's supported languages list](supported_languages.md):
   - If a translation for that language **already exists** in the file, it is **skipped** (never overwritten).
   - Otherwise, it calls the Google Translate API to translate the source text into that language and adds it to the
     file.
3. The file is rewritten in place with all the new translations added.

Example:

```bash
slu resource translate i18n/display_name.yml
```

```
[01/68] English (Afrikaans)          => "Speel Rekenaar"
[02/68] Amharic                      => "ጨዋታ ኮምፒተር"
...
[14/68] English                      => Skipped
...
```

If your source text is not in English, pass the language code explicitly:

```bash
slu resource translate i18n/description.yml es
```

### Practical workflow

- Run the command **once per file** you want translated — it does not recurse into a folder or a whole `metadata.yml`.
  If you split translations across many files under `i18n/` (as recommended in
  [section 3](#3-writing-resourcetext-in-yaml)), run it once for each.
- Because existing entries are skipped, it's safe to run the command again later after adding a new language by hand or
  fixing a bad machine translation — it will only fill in what's still missing.
- **Always review the output.** Machine translation from Google Translate can produce awkward or outright wrong
  phrasing, especially for short UI labels with little context. Treat the generated file as a first draft, not a final
  translation.
- Do not run this command as part of routine iteration — treat it like a one-time step per file, right before you
  bundle/publish the resource, since re-running it repeatedly against a live Google Translate backend is wasteful and
  unnecessary once a language is filled in.

---

## 5. Tips and Gotchas

- **`en` is mandatory** once you use the map form. A resource whose `displayName` or `description` map has no `en` key
  fails `Resource::verify` and cannot be published.
- **Plain string is shorthand for `en`.** `label: My Setting` and `label: { en: My Setting }` are equivalent — the CLI
  and the UI treat them identically.
- **`zh` and `pt` are not valid target codes** — Seelen UI only ships `zh-CN`/`zh-TW` and `pt-BR`/`pt-PT`. The translate
  command already maps these internally when calling Google Translate, so you don't need to worry about it, but don't
  hand-write a bare `zh:` or `pt:` key yourself — it will never match the running app's language and will silently never
  be shown.
- **Missing text renders as nothing.** If neither the current language nor `en` has an entry, the resource text renders
  as an empty span in Settings (or `!?` if read directly on the Rust side) — always keep `en` filled in.
