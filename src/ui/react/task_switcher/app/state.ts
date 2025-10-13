import { batch, effect, signal } from "@preact/signals";
import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { listen } from "@tauri-apps/api/event";

export const $windows = signal(await invoke(SeelenCommand.GetUserAppWindows));
subscribe(SeelenEvent.UserAppWindowsChanged, (e) => ($windows.value = e.payload));

export const $focusedWinId = signal((await invoke(SeelenCommand.GetFocusedApp)).hwnd);
subscribe(SeelenEvent.GlobalFocusChanged, (e) => ($focusedWinId.value = e.payload.hwnd));

export const $selectedWindow = signal<number | null>(null);
effect(() => {
  const win = $windows.value.find((w) => w.hwnd === $focusedWinId.value);
  $selectedWindow.value = win?.hwnd || null;
});

export const $showing = signal(false);
export const $autoConfirm = signal(false);

listen<boolean>("hidden::task-switcher-select-next", ({ payload: autoConfirm }) => {
  batch(() => {
    $showing.value = true;
    $autoConfirm.value = autoConfirm;
  });
  // cycle next index, go to first if last.
  const index = $windows.value.findIndex((w) => w.hwnd === $selectedWindow.value);
  $selectedWindow.value = $windows.value[(index + 1) % $windows.value.length]?.hwnd || null;
});
listen<boolean>("hidden::task-switcher-select-previous", ({ payload: autoConfirm }) => {
  batch(() => {
    $showing.value = true;
    $autoConfirm.value = autoConfirm;
  });
  // cycle previous index, go to last if first.
  const index = $windows.value.findIndex((w) => w.hwnd === $selectedWindow.value);
  $selectedWindow.value = $windows.value[(index - 1 + $windows.value.length) % $windows.value.length]?.hwnd || null;
});

let widget = Widget.getCurrent();
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
    $showing.value = true;
  }
};
