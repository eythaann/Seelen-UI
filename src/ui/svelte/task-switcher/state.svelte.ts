import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import type { UserAppWindow } from "@seelen-ui/lib/types";
import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";

const widget = Widget.getCurrent();

type WindowFilterMode = "all" | "blacklist" | "whitelist";

interface TaskSwitcherSettings {
  windowFilterMode: WindowFilterMode;
  windowFilterPatterns: string;
  previewEnabled: boolean;
  previewDelay: number;
}

function getTaskSwitcherSettings(settings: Settings): TaskSwitcherSettings {
  const config = settings.getCurrentWidgetConfig();
  const mode = config.windowFilterMode;
  const delay = Number(config.previewDelay);

  return {
    windowFilterMode: mode === "blacklist" || mode === "whitelist" ? mode : "all",
    windowFilterPatterns: typeof config.windowFilterPatterns === "string" ? config.windowFilterPatterns : "",
    previewEnabled: typeof config.previewEnabled === "boolean" ? config.previewEnabled : true,
    previewDelay: Number.isFinite(delay) ? Math.min(5000, Math.max(0, delay)) : 1000,
  };
}

function filterWindows(
  windows: UserAppWindow[],
  { windowFilterMode, windowFilterPatterns }: TaskSwitcherSettings,
): UserAppWindow[] {
  const patterns = windowFilterPatterns
    .split(/\r?\n/)
    .map((pattern) => pattern.trim().toLowerCase())
    .filter(Boolean);

  if (windowFilterMode === "all" || patterns.length === 0) {
    return windows;
  }

  return windows.filter((window) => {
    const executablePath = window.process.path?.toLowerCase() ?? "";
    const executableName = executablePath.split(/[\\/]/).pop() ?? "";
    const searchableValues = [
      window.appName.toLowerCase(),
      window.title.toLowerCase(),
      executableName,
      executablePath,
    ];
    const matches = patterns.some((pattern) => searchableValues.some((value) => value.includes(pattern)));

    return windowFilterMode === "whitelist" ? matches : !matches;
  });
}

// +++++++++++++++++++++++ Reactive State +++++++++++++++++++++++

let showing = $state(false);
let autoConfirm = $state(false);

let settings = lazyRune(async () => getTaskSwitcherSettings(await Settings.getAsync()));

let windows = lazyRune(async () =>
  (await invoke(SeelenCommand.GetUserAppWindows)).toSorted(
    (a, b) => b.lastForegroundAt - a.lastForegroundAt,
  )
);
subscribe(SeelenEvent.UserAppWindowsChanged, ({ payload }) => {
  windows.value = payload.toSorted((a, b) => b.lastForegroundAt - a.lastForegroundAt);
});

let previews = lazyRune(async () =>
  settings.value.previewEnabled ? await invoke(SeelenCommand.GetUserAppWindowsPreviews) : {}
);
subscribe(SeelenEvent.UserAppWindowsPreviewsChanged, (event) => {
  if (settings.isInitialized() && settings.value.previewEnabled) {
    previews.setByPayload(event);
  }
});

async function refreshPreviews() {
  const nextPreviews = await invoke(SeelenCommand.GetUserAppWindowsPreviews);
  if (settings.value.previewEnabled) {
    previews.value = nextPreviews;
  }
}

Settings.onChange((nextSettings) => {
  const wasPreviewEnabled = settings.isInitialized() && settings.value.previewEnabled;
  settings.value = getTaskSwitcherSettings(nextSettings);

  if (!settings.value.previewEnabled && previews.isInitialized()) {
    previews.value = {};
  } else if (settings.value.previewEnabled && !wasPreviewEnabled && previews.isInitialized()) {
    void refreshPreviews();
  }
});

let focusedWinId = lazyRune(async () => (await invoke(SeelenCommand.GetFocusedApp)).hwnd);
subscribe(SeelenEvent.GlobalFocusChanged, (e) => {
  focusedWinId.value = e.payload.hwnd;
});

let monitors = lazyRune(() => invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, monitors.setByPayload);

await settings.init();
await Promise.all([windows.init(), previews.init(), focusedWinId.init(), monitors.init()]);

let filteredWindows = $derived.by(() => filterWindows(windows.value, settings.value));
let selectedWindow = $state<number | null>(focusedWinId.value ?? null);

// Keep the selection inside the filtered window set as windows or settings change.
$effect.root(() => {
  $effect(() => {
    if (!showing) {
      const win = filteredWindows.find((w) => w.hwnd === focusedWinId.value);
      selectedWindow = win?.hwnd ?? null;
    } else if (!filteredWindows.some((window) => window.hwnd === selectedWindow)) {
      selectedWindow = filteredWindows[0]?.hwnd ?? null;
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
    return filteredWindows;
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

  get previewEnabled() {
    return settings.value.previewEnabled;
  }

  get previewDelay() {
    return settings.value.previewDelay;
  }
}

export const globalState = new State();

// +++++++++++++++++++++++ Visibility +++++++++++++++++++++++

$effect.root(() => {
  $effect(() => {
    let cancelled = false;

    if (showing) {
      widget.show().then(async () => {
        if (cancelled) {
          return;
        }

        // double check for fast keyboard trigger
        let isPressing = await invoke(SeelenCommand.GetKeyState, { key: "Alt" });
        if (isPressing) {
          await widget.focus();
        } else {
          onAltKeyUp();
        }
      });
    } else {
      widget.hide();
    }

    return () => {
      cancelled = true;
    };
  });

  // Hide when focus leaves the widget
  $effect(() => {
    if (focusedWinId.value !== widget.windowId) {
      showing = false;
    }
  });
});

// +++++++++++++++++++++++ Triggering +++++++++++++++++++++++

function onAltKeyUp() {
  if (showing && autoConfirm) {
    showing = false;
    if (selectedWindow) {
      invoke(SeelenCommand.WegToggleWindowState, {
        hwnd: selectedWindow,
        wasFocused: false,
      });
    }
  }
}

widget.onTrigger((payload) => {
  const direction: string = (payload.customArgs?.direction as string) || "next";
  const autoConfirmValue: boolean = (payload.customArgs?.autoConfirm as boolean) || false;
  const availableWindows = filteredWindows;

  if (availableWindows.length === 0) {
    return;
  }

  // Use the currently selected window when already showing, otherwise start from focused
  const currentHwnd = showing ? selectedWindow : focusedWinId.value;

  let index = availableWindows.findIndex((w) => w.hwnd === currentHwnd);
  if (direction === "next") {
    if (index === -1) index = availableWindows.length - 1;
    selectedWindow = availableWindows[(index + 1) % availableWindows.length]?.hwnd ?? null;
  } else if (direction === "previous") {
    if (index === -1) index = 0;
    selectedWindow = availableWindows[(index - 1 + availableWindows.length) % availableWindows.length]?.hwnd ?? null;
  }

  // Only capture autoConfirm on the first trigger (when switcher was hidden)
  if (!showing) {
    autoConfirm = autoConfirmValue;
  }
  showing = true;
});

window.onkeydown = (e) => {
  if (e.key === "Escape") {
    showing = false;
  }
};

window.onkeyup = (e) => {
  if (e.key === "Alt") {
    onAltKeyUp();
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
