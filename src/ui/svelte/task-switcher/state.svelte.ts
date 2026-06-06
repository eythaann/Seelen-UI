import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";

const widget = Widget.getCurrent();

// +++++++++++++++++++++++ Reactive State +++++++++++++++++++++++

let showing = $state(false);
let autoConfirm = $state(false);

let windows = lazyRune(async () =>
  (await invoke(SeelenCommand.GetUserAppWindows)).toSorted(
    (a, b) => b.lastForegroundAt - a.lastForegroundAt,
  )
);
subscribe(SeelenEvent.UserAppWindowsChanged, ({ payload }) => {
  windows.value = payload.toSorted((a, b) => b.lastForegroundAt - a.lastForegroundAt);
});

let previews = lazyRune(() => invoke(SeelenCommand.GetUserAppWindowsPreviews));
subscribe(SeelenEvent.UserAppWindowsPreviewsChanged, previews.setByPayload);

let focusedWinId = lazyRune(async () => (await invoke(SeelenCommand.GetFocusedApp)).hwnd);
subscribe(SeelenEvent.GlobalFocusChanged, (e) => {
  focusedWinId.value = e.payload.hwnd;
});

let monitors = lazyRune(() => invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, monitors.setByPayload);

await Promise.all([windows.init(), previews.init(), focusedWinId.init(), monitors.init()]);

let selectedWindow = $state<number | null>(focusedWinId.value ?? null);

// Sync selectedWindow with focused window when the switcher is not visible
$effect.root(() => {
  $effect(() => {
    if (!showing) {
      const win = windows.value.find((w) => w.hwnd === focusedWinId.value);
      selectedWindow = win?.hwnd ?? null;
    }
  });
});

// +++++++++++++++++++++++ State Class +++++++++++++++++++++++

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
}

export const globalState = new State();

// +++++++++++++++++++++++ Visibility +++++++++++++++++++++++

$effect.root(() => {
  $effect(() => {
    if (showing) {
      widget.show().then(() => widget.focus());
    } else {
      widget.hide();
    }
  });
});

// Hide when focus leaves the widget; dispatch synthetic Alt keyup if Alt was released.
// Uses a cancellation flag to discard responses from stale async checks.
$effect.root(() => {
  $effect(() => {
    let isFocused = focusedWinId.value === widget.windowId;
    let cancelled = false;

    if (isFocused) {
      invoke(SeelenCommand.GetKeyState, { key: "Alt" }).then((isPressing) => {
        if (!cancelled && !isPressing) {
          window.dispatchEvent(new KeyboardEvent("keyup", { key: "Alt" }));
        }
      });
    } else {
      showing = false;
    }

    return () => {
      cancelled = true;
    };
  });
});

// +++++++++++++++++++++++ Triggering +++++++++++++++++++++++

widget.onTrigger((payload) => {
  const direction: string = (payload.customArgs?.direction as string) || "next";
  const autoConfirmValue: boolean = (payload.customArgs?.autoConfirm as boolean) || false;

  if (windows.value.length === 0) {
    return;
  }

  // Use the currently selected window when already showing, otherwise start from focused
  const currentHwnd = showing ? selectedWindow : focusedWinId.value;

  // Only capture autoConfirm on the first trigger (when switcher was hidden)
  if (!showing) {
    autoConfirm = autoConfirmValue;
  }
  showing = true;

  let index = windows.value.findIndex((w) => w.hwnd === currentHwnd);
  if (direction === "next") {
    if (index === -1) index = windows.value.length - 1;
    selectedWindow = windows.value[(index + 1) % windows.value.length]?.hwnd ?? null;
  } else if (direction === "previous") {
    if (index === -1) index = 0;
    selectedWindow = windows.value[(index - 1 + windows.value.length) % windows.value.length]?.hwnd ?? null;
  }
});

window.onkeydown = (e) => {
  if (e.key === "Escape") {
    showing = false;
  }
};

window.onkeyup = (e) => {
  if (e.key === "Alt" && showing && selectedWindow && autoConfirm) {
    showing = false;
    invoke(SeelenCommand.WegToggleWindowState, {
      hwnd: selectedWindow,
      wasFocused: false,
    });
  }
};

// +++++++++++++++++++++++ Sizing +++++++++++++++++++++++

let primaryMonitor = $derived.by(
  () => monitors.value.find((m) => m.isPrimary) || monitors.value[0],
);

$effect.root(() => {
  $effect(() => {
    if (primaryMonitor) {
      widget.setPosition(primaryMonitor.rect);
    }
  });
});
