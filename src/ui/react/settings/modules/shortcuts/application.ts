import type { SluHotkey, SluShortcutsSettings } from "@seelen-ui/lib/types";
import { signal } from "@preact/signals";
import { settings } from "../../state/mod";
import { Settings } from "@seelen-ui/lib";
import type { WidgetId } from "@seelen-ui/lib/types";

export const shortcutsError = signal<Set<string>>(new Set());

// Load default settings for reset functionality
const defaultSettings = await Settings.default();

interface Groups {
  windowManager: {
    state: SluHotkey[];
    sizing: SluHotkey[];
    positioning: SluHotkey[];
    tilingFocus: SluHotkey[];
    tilingLayout: SluHotkey[];
  };
  wallpaperManager: SluHotkey[];
  virtualDesktop: {
    main: SluHotkey[];
    switch: SluHotkey[];
    move: SluHotkey[];
    send: SluHotkey[];
  };
  weg: SluHotkey[];
  misc: SluHotkey[];
}

export function getHotkeysGroups(hotkeys: SluHotkey[]): Groups {
  const groups: Groups = {
    windowManager: {
      state: [],
      sizing: [],
      positioning: [],
      tilingFocus: [],
      tilingLayout: [],
    },
    wallpaperManager: [],
    virtualDesktop: {
      main: [],
      switch: [],
      move: [],
      send: [],
    },
    weg: [],
    misc: [],
  };

  for (const hotkey of hotkeys) {
    const {
      action: { name },
    } = hotkey;

    switch (name) {
      // window manager
      case "increase_width":
      case "decrease_width":
      case "increase_height":
      case "decrease_height":
      case "restore_sizes":
        groups.windowManager.sizing.push(hotkey);
        break;
      case "focus_top":
      case "focus_bottom":
      case "focus_left":
      case "focus_right":
        groups.windowManager.tilingFocus.push(hotkey);
        break;
      case "move_window_left":
      case "move_window_right":
      case "move_window_up":
      case "move_window_down":
        groups.windowManager.positioning.push(hotkey);
        break;
      case "reserve_left":
      case "reserve_right":
      case "reserve_top":
      case "reserve_bottom":
      case "reserve_stack":
      case "reserve_float":
        groups.windowManager.tilingLayout.push(hotkey);
        break;
      case "pause_tiling":
      case "toggle_float":
      case "toggle_monocle":
      case "cycle_stack_next":
      case "cycle_stack_prev":
        groups.windowManager.state.push(hotkey);
        break;
      // weg
      case "start_weg_app":
        groups.weg.push(hotkey);
        break;
      // wallpaper manager
      case "cycle_wallpaper_next":
      case "cycle_wallpaper_prev":
        groups.wallpaperManager.push(hotkey);
        break;
      // virtual desktop
      case "create_new_workspace":
      case "destroy_current_workspace":
      case "switch_to_next_workspace":
      case "switch_to_previous_workspace":
        groups.virtualDesktop.main.push(hotkey);
        break;
      case "switch_workspace":
        groups.virtualDesktop.switch.push(hotkey);
        break;
      case "move_to_workspace":
        groups.virtualDesktop.move.push(hotkey);
        break;
      case "send_to_workspace":
        groups.virtualDesktop.send.push(hotkey);
        break;
      // misc
      case "misc_open_settings":
      case "misc_toggle_lock_tracing":
      case "misc_toggle_win_event_tracing":
      case "misc_force_restart":
      case "misc_force_quit":
        groups.misc.push(hotkey);
        break;
      default:
        break;
    }
  }

  return groups;
}

export function validateShortcuts(hotkeys: SluHotkey[]) {
  const errors = new Set<string>();
  const shortcutMap = new Map<string, string[]>();

  for (const hotkey of hotkeys) {
    if (hotkey.readonly || hotkey.system) {
      continue;
    }

    const shortcutKey = hotkey.keys.join("+").toLowerCase();
    if (!shortcutMap.has(shortcutKey)) {
      shortcutMap.set(shortcutKey, []);
    }
    shortcutMap.get(shortcutKey)!.push(hotkey.id);
  }

  for (const [, ids] of shortcutMap) {
    if (ids.length > 1) {
      ids.forEach((id) => errors.add(id));
    }
  }

  shortcutsError.value = errors;
}

/**
 * Gets the current shortcuts configuration
 */
export function getShortcutsConfig(): SluShortcutsSettings {
  return settings.value.shortcuts;
}

/**
 * Sets the enabled state for shortcuts
 */
export function setShortcutsEnabled(enabled: boolean) {
  settings.value = {
    ...settings.value,
    shortcuts: {
      ...settings.value.shortcuts,
      enabled,
    },
  };
}

/**
 * Updates a specific shortcut by id
 */
export function updateShortcut(id: string, keys: string[]) {
  const appCommands = settings.value.shortcuts.appCommands.map((c) => c.id === id ? { ...c, keys } : c);

  settings.value = {
    ...settings.value,
    shortcuts: {
      ...settings.value.shortcuts,
      appCommands,
    },
  };
}

/**
 * Resets all shortcuts to default values
 */
export function resetShortcuts() {
  settings.value = {
    ...settings.value,
    shortcuts: {
      enabled: settings.value.shortcuts.enabled,
      appCommands: structuredClone(defaultSettings.shortcuts.appCommands),
    },
  };
}

/**
 * Checks if a widget is enabled
 */
export function isWidgetEnabled(widgetId: WidgetId): boolean {
  const widget = settings.value.byWidget[widgetId];
  return !!widget?.enabled;
}
