import type { SluHotkey } from "@seelen-ui/lib/types";

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
