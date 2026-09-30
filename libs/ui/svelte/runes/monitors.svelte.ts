import { lazyRune } from "../utils";
import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { Rect } from "@seelen-ui/lib/types";
import { derivedWith } from "../utils/derivedWith.svelte";

let _monitors = lazyRune(() => invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, _monitors.setByPayload);

const primaryMonitor = derivedWith(
  () => _monitors.value.find((m) => m.isPrimary) || _monitors.value[0],
);

const desktopRect = derivedWith(() => {
  let rect = { top: 0, left: 0, right: 0, bottom: 0 };
  for (const monitor of _monitors.value) {
    rect.left = Math.min(rect.left, monitor.rect.left);
    rect.top = Math.min(rect.top, monitor.rect.top);
    rect.right = Math.max(rect.right, monitor.rect.right);
    rect.bottom = Math.max(rect.bottom, monitor.rect.bottom);
  }
  return rect;
});

class MonitorState {
  init(): Promise<void> {
    return _monitors.init();
  }

  get all() {
    return _monitors.value;
  }

  get primaryMonitor() {
    return primaryMonitor.value;
  }

  get desktopRect() {
    return desktopRect.value;
  }

  calculateDesktopRelativeRect(rect: Rect): Rect {
    return {
      left: rect.left - desktopRect.value.left,
      top: rect.top - desktopRect.value.top,
      right: rect.right - desktopRect.value.left,
      bottom: rect.bottom - desktopRect.value.top,
    };
  }
}

export const monitors = new MonitorState();
