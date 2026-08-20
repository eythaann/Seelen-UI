import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";
import z from "zod";

const widget = Widget.getCurrent();

const WidgetConfigSchema = z.object({
  onlyOnActiveMonitor: z.boolean(),
});

let settings = lazyRune(() => Settings.getAsync());
await Settings.onChange((s) => (settings.value = s));
await settings.init();

const widgetConfig = $derived.by(
  () =>
    WidgetConfigSchema.safeParse(settings.value.getCurrentWidgetConfig()).data ??
      (widget.getDefaultConfig() as unknown as z.infer<typeof WidgetConfigSchema>),
);

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

let desiredPosition = $state<{ x: number; y: number } | null>(null);

await Promise.all([windows.init(), previews.init(), focusedWinId.init(), monitors.init()]);

let selectedWindow = $state<number | null>(focusedWinId.value ?? null);

// Sync selectedWindow with focused window when the switcher is not visible
$effect.root(() => {
  $effect(() => {
    if (!showing) {
      const win = filteredWindows.find((w) => w.hwnd === focusedWinId.value);
      selectedWindow = win?.hwnd ?? null;
    }
  });
});

let desktopRect = $derived.by(() => {
  let rect = { top: 0, left: 0, right: 0, bottom: 0 };
  for (const monitor of monitors.value) {
    rect.left = Math.min(rect.left, monitor.rect.left);
    rect.top = Math.min(rect.top, monitor.rect.top);
    rect.right = Math.max(rect.right, monitor.rect.right);
    rect.bottom = Math.max(rect.bottom, monitor.rect.bottom);
  }
  return rect;
});

// Monitor under the cursor position that triggered the switcher, falling back to primary
let activeMonitor = $derived.by(() => {
  const pos = desiredPosition;
  const found = pos &&
    monitors.value.find(
      (m) => m.rect.left <= pos.x && pos.x < m.rect.right && m.rect.top <= pos.y && pos.y < m.rect.bottom,
    );
  return found || monitors.value.find((m) => m.isPrimary) || monitors.value[0];
});

// Windows shown in the switcher, optionally restricted to the active monitor
let filteredWindows = $derived.by(() => {
  const monitor = activeMonitor;
  if (!widgetConfig.onlyOnActiveMonitor || !monitor) {
    return windows.value;
  }
  return windows.value.filter((w) => w.monitor === monitor.id);
});

let relativeActiveMonitor = $derived.by(() => {
  const monitor = activeMonitor;
  if (!monitor) {
    return null;
  }
  return {
    ...monitor,
    rect: {
      top: monitor.rect.top - desktopRect.top,
      left: monitor.rect.left - desktopRect.left,
      right: monitor.rect.right - desktopRect.left,
      bottom: monitor.rect.bottom - desktopRect.top,
    },
  };
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

  get activeMonitor() {
    return relativeActiveMonitor;
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
  if (showing && selectedWindow && autoConfirm) {
    showing = false;
    invoke(SeelenCommand.WegToggleWindowState, {
      hwnd: selectedWindow,
      wasFocused: false,
    });
  }
}

widget.onTrigger((payload) => {
  const direction: string = (payload.customArgs?.direction as string) || "next";
  const autoConfirmValue: boolean = (payload.customArgs?.autoConfirm as boolean) || false;

  // Only capture autoConfirm and the trigger monitor on the first trigger (when switcher was hidden),
  // and do it before filtering windows so the monitor filter reflects the new cursor position.
  if (!showing) {
    autoConfirm = autoConfirmValue;
    if (payload.desiredPosition) {
      desiredPosition = payload.desiredPosition;
    }
  }

  const targetWindows = filteredWindows;
  if (targetWindows.length === 0) {
    return;
  }

  // Use the currently selected window when already showing, otherwise start from focused
  const currentHwnd = showing ? selectedWindow : focusedWinId.value;

  let index = targetWindows.findIndex((w) => w.hwnd === currentHwnd);
  if (direction === "next") {
    if (index === -1) index = targetWindows.length - 1;
    selectedWindow = targetWindows[(index + 1) % targetWindows.length]?.hwnd ?? null;
  } else if (direction === "previous") {
    if (index === -1) index = 0;
    selectedWindow = targetWindows[(index - 1 + targetWindows.length) % targetWindows.length]?.hwnd ?? null;
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

// The window itself spans all monitors (like the power-menu overlay); the switcher
// content is then positioned onto whichever monitor the cursor was on at trigger time.
$effect.root(() => {
  $effect(() => {
    widget.setPosition(desktopRect);
  });
});
