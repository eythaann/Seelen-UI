import type { ResourceText, SluShortcutsSettings, Widget, WidgetShortcutDeclaration } from "@seelen-ui/lib/types";
import { signal } from "@preact/signals";
import { settings } from "../../state/mod";
import { lazySignal } from "libs/ui/react/utils/LazySignal";
import { invoke, SeelenCommand } from "@seelen-ui/lib";
import type { WidgetId } from "@seelen-ui/lib/types";
import { cloneDeep } from "lodash";
import { widgets } from "../../state/resources";

export const shortcutsError = signal<Set<string>>(new Set());
export const systemShortcuts = lazySignal(() => invoke(SeelenCommand.StateGetSystemShortcuts));
await systemShortcuts.init();

// ─── Types ───────────────────────────────────────────────────────────────────

/** A shortcut as shown in the settings UI — declaration merged with current effective keys. */
export interface ShortcutEntry {
  id: string;
  label: ResourceText | string;
  keys: string[];
  defaultKeys: string[];
  readonly: boolean;
  /** Widget ID if widget-owned, null if system-level. */
  widgetId: WidgetId | null;
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

function getWidgetOverrides(widgetId: WidgetId): Record<string, string[]> {
  const byWidget = settings.value.byWidget;
  return byWidget[widgetId]?.$shortcuts ?? {};
}

function widgetShortcutToEntry(decl: WidgetShortcutDeclaration, widgetId: WidgetId): ShortcutEntry {
  const overrides = getWidgetOverrides(widgetId);

  return {
    id: decl.id,
    label: decl.label,
    keys: decl.readonly ? decl.defaultKeys : (overrides[decl.id] ?? decl.defaultKeys),
    defaultKeys: decl.defaultKeys,
    readonly: decl.readonly,
    widgetId,
  };
}

// ─── Public API ──────────────────────────────────────────────────────────────

export function getShortcutsConfig(): SluShortcutsSettings {
  return settings.value.shortcuts;
}

export function setShortcutsEnabled(enabled: boolean) {
  settings.value = { ...settings.value, shortcuts: { ...settings.value.shortcuts, enabled } };
}

export function updateShortcut(entry: ShortcutEntry, keys: string[]) {
  if (entry.widgetId) {
    const byWidget = settings.value.byWidget;

    settings.value = {
      ...settings.value,
      byWidget: {
        ...settings.value.byWidget,
        [entry.widgetId]: {
          ...(byWidget[entry.widgetId] ?? {}),
          $shortcuts: { ...getWidgetOverrides(entry.widgetId), [entry.id]: keys },
        },
      } as typeof settings.value.byWidget,
    };
  } else {
    settings.value = {
      ...settings.value,
      shortcuts: {
        ...settings.value.shortcuts,
        shortcuts: { ...settings.value.shortcuts.shortcuts, [entry.id]: keys },
      },
    };
  }
}

export function resetShortcuts() {
  const byWidget = cloneDeep(settings.value.byWidget);
  for (const value of Object.values(byWidget)) {
    value!.$shortcuts = null;
  }

  settings.value = {
    ...settings.value,
    shortcuts: { ...settings.value.shortcuts, shortcuts: {} },
    byWidget: { ...settings.value.byWidget, ...byWidget },
  };
}

export function validateShortcuts(entries: ShortcutEntry[]) {
  const errors = new Set<string>();
  const seen = new Map<string, string>();
  for (const entry of entries) {
    if (entry.readonly || entry.keys.length === 0) continue;
    const key = entry.keys.join("+").toLowerCase();
    if (seen.has(key)) {
      errors.add(seen.get(key)!);
      errors.add(entry.id);
    } else {
      seen.set(key, entry.id);
    }
  }
  shortcutsError.value = errors;
}

// ─── Grouped entries for the UI ──────────────────────────────────────────────

export interface ShortcutGroups {
  byWidget: Map<WidgetId, { widget: Widget; entries: ShortcutEntry[] }>;
  system: {
    vdMain: ShortcutEntry[];
    vdSwitch: ShortcutEntry[];
    vdMove: ShortcutEntry[];
    vdSend: ShortcutEntry[];
    misc: ShortcutEntry[];
  };
}

export function getShortcutGroups(): ShortcutGroups {
  const byWidget = new Map<WidgetId, { widget: Widget; entries: ShortcutEntry[] }>();

  for (const widget of widgets.value) {
    if (!widget.shortcuts.length) continue;
    const entries = widget.shortcuts.map((decl) => widgetShortcutToEntry(decl, widget.id));
    byWidget.set(widget.id, { widget, entries });
  }

  const systemOverrides = settings.value.shortcuts.shortcuts;
  const systemEntries = systemShortcuts.value.map(
    (d): ShortcutEntry => ({
      id: d.id,
      label: d.label,
      keys: d.readonly ? d.defaultKeys : (systemOverrides[d.id] ?? d.defaultKeys),
      defaultKeys: d.defaultKeys,
      readonly: d.readonly,
      widgetId: null,
    }),
  );

  const system: ShortcutGroups["system"] = {
    vdMain: systemEntries.filter((e) =>
      ["vd-switch-next", "vd-switch-prev", "vd-create-workspace", "vd-destroy-workspace"].includes(
        e.id,
      )
    ),
    vdSwitch: systemEntries.filter((e) => e.id.startsWith("vd-switch-to-")),
    vdMove: systemEntries.filter((e) => e.id.startsWith("vd-move-to-")),
    vdSend: systemEntries.filter((e) => e.id.startsWith("vd-send-to-")),
    misc: systemEntries.filter(
      (e) =>
        ![
          "vd-switch-next",
          "vd-switch-prev",
          "vd-create-workspace",
          "vd-destroy-workspace",
        ].includes(e.id) &&
        !e.id.startsWith("vd-switch-to-") &&
        !e.id.startsWith("vd-move-to-") &&
        !e.id.startsWith("vd-send-to-") &&
        !e.id.startsWith("weg-launch-"),
    ),
  };

  return { byWidget, system };
}
