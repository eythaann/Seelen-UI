import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";

let widget = Widget.getCurrent();

let showing = $state(false);
let autoConfirm = $state(false);

let windows = lazyRune(() => invoke(SeelenCommand.GetUserAppWindows));
await subscribe(SeelenEvent.UserAppWindowsChanged, windows.setByPayload);
await windows.init();

let previews = lazyRune(() => invoke(SeelenCommand.GetUserAppWindowsPreviews));
await subscribe(SeelenEvent.UserAppWindowsPreviewsChanged, previews.setByPayload);
await previews.init();

let focusedWinId = lazyRune(async () => (await invoke(SeelenCommand.GetFocusedApp)).hwnd);
await subscribe(SeelenEvent.GlobalFocusChanged, (e) => {
  focusedWinId.value = e.payload.hwnd;
});
await focusedWinId.init();

let selectedWindow = $state<number | null>(focusedWinId.value);

// Only sync with focused window when task switcher is hidden
$effect.root(() => {
  $effect(() => {
    if (!showing) {
      const win = windows.value.find((w) => w.hwnd === focusedWinId.value);
      selectedWindow = win?.hwnd || null;
    }
  });
});

class State {
  get showing() {
    return showing;
  }

  set showing(value: boolean) {
    showing = value;
  }

  get windows() {
    return windows.value;
  }

  get previews() {
    return previews.value;
  }

  get selectedWindow() {
    return selectedWindow;
  }
  set selectedWindow(value: number | null) {
    selectedWindow = value;
  }

  /**
   * Optimistically moves the selected window to the front of the array
   * for fast UI responsiveness. The backend will send the updated order
   * via events, but this provides immediate visual feedback.
   */
  moveSelectedToFront(hwnd: number) {
    const currentWindows = windows.value;
    const selectedIndex = currentWindows.findIndex((w) => w.hwnd === hwnd);

    if (selectedIndex > 0) {
      // Only reorder if not already at the front
      const reordered = [
        currentWindows[selectedIndex]!,
        ...currentWindows.slice(0, selectedIndex),
        ...currentWindows.slice(selectedIndex + 1),
      ];
      windows.value = reordered;
    }
  }
}

export const globalState = new State();

// +++++++++++++++++++++++ Triggering +++++++++++++++++++++++

$effect.root(() => {
  $effect(() => {
    if (showing) {
      widget.webview.show().then(() => widget.webview.setFocus());
    } else {
      widget.webview.hide();
    }
  });
});

widget.onTrigger((payload) => {
  const direction: string = (payload.customArgs?.direction as string) || "next";
  const autoConfirmValue: boolean = (payload.customArgs?.autoConfirm as boolean) || false;

  // Don't show if there are no windows
  if (windows.value.length === 0) {
    return;
  }

  // If switcher was hidden, use focused window as starting point
  const currentHwnd = showing ? selectedWindow : focusedWinId.value;

  // Only set autoConfirm on first show (when switcher was hidden)
  if (!showing) {
    autoConfirm = autoConfirmValue;
  }
  showing = true;

  let index = windows.value.findIndex((w) => w.hwnd === currentHwnd);
  if (direction === "next") {
    if (index === -1) {
      index = windows.value.length - 1; // Will cycle to 0 with (index + 1) % length
    }
    selectedWindow = windows.value[(index + 1) % windows.value.length]?.hwnd || null;
  } else if (direction === "previous") {
    if (index === -1) {
      index = 0; // Will cycle to last with (index - 1 + length) % length
    }
    selectedWindow = windows.value[(index - 1 + windows.value.length) % windows.value.length]?.hwnd || null;
  }
});

widget.webview.onFocusChanged(({ payload }) => {
  if (!payload) {
    showing = false;
  }
});

window.onkeyup = (e) => {
  if (e.key === "Alt" && selectedWindow && autoConfirm) {
    showing = false;
    invoke(SeelenCommand.WegToggleWindowState, {
      hwnd: selectedWindow,
      wasFocused: false,
    });
    // Optimistically reorder UI before backend updates
    globalState.moveSelectedToFront(selectedWindow);
  }
};

// +++++++++++++++++++++++ Sizing +++++++++++++++++++++++

let monitors = lazyRune(() => invoke(SeelenCommand.SystemGetMonitors));
await subscribe(SeelenEvent.SystemMonitorsChanged, monitors.setByPayload);
await monitors.init();

let primaryMonitor = $derived.by(() => {
  return monitors.value.find((m) => m.isPrimary) || monitors.value[0];
});

$effect.root(() => {
  $effect(() => {
    if (primaryMonitor) {
      Widget.getCurrent().setPosition(primaryMonitor.rect);
    }
  });
});
