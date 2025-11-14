import { batch, effect, signal } from "@preact/signals";
import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";

let widget = Widget.getCurrent();

export const $showing = signal(false);
export const $autoConfirm = signal(false);

export const $windows = signal(await invoke(SeelenCommand.GetUserAppWindows));
subscribe(SeelenEvent.UserAppWindowsChanged, (e) => ($windows.value = e.payload));

export const $focusedWinId = signal((await invoke(SeelenCommand.GetFocusedApp)).hwnd);
subscribe(SeelenEvent.GlobalFocusChanged, (e) => ($focusedWinId.value = e.payload.hwnd));

export const $selectedWindow = signal<number | null>($focusedWinId.value);

// Only sync with focused window when task switcher is hidden
effect(() => {
  if (!$showing.value) {
    const win = $windows.value.find((w) => w.hwnd === $focusedWinId.value);
    $selectedWindow.value = win?.hwnd || null;
  }
});

widget.onTrigger((payload) => {
  const direction: string = (payload.customArgs?.direction as string) || "next";
  const autoConfirm: boolean = (payload.customArgs?.autoConfirm as boolean) || false;

  // Don't show if there are no windows
  if ($windows.value.length === 0) {
    return;
  }

  const wasShowing = $showing.value;
  batch(() => {
    // Only set autoConfirm on first show (when switcher was hidden)
    if (!wasShowing) {
      $autoConfirm.value = autoConfirm;
    }
    $showing.value = true;
  });

  // If switcher was hidden, use focused window as starting point
  const currentHwnd = wasShowing ? $selectedWindow.value : $focusedWinId.value;

  let index = $windows.value.findIndex((w) => w.hwnd === currentHwnd);
  if (direction === "next") {
    if (index === -1) {
      index = $windows.value.length - 1; // Will cycle to 0 with (index + 1) % length
    }
    $selectedWindow.value = $windows.value[(index + 1) % $windows.value.length]?.hwnd || null;
  } else if (direction === "previous") {
    if (index === -1) {
      index = 0; // Will cycle to last with (index - 1 + length) % length
    }
    $selectedWindow.value = $windows.value[(index - 1 + $windows.value.length) % $windows.value.length]?.hwnd || null;
  }
});

$showing.subscribe(async (show) => {
  if (show) {
    await widget.webview.show();
    await widget.webview.setFocus();
  } else {
    await widget.webview.hide();
  }
});

widget.webview.onFocusChanged(({ payload }) => {
  if (!payload) {
    $showing.value = false;
  }
});

window.onkeyup = (e) => {
  if (e.key === "Alt" && $selectedWindow.value && $autoConfirm.value) {
    invoke(SeelenCommand.WegToggleWindowState, {
      hwnd: $selectedWindow.value,
      wasFocused: false,
    });
    $showing.value = false;
  }
};
