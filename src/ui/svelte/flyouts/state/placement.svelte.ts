import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { lazyRune } from "libs/ui/svelte/utils";
import { ConfigState } from "./config.svelte";
import { PhysicalPosition } from "@tauri-apps/api/window";

let showing = $state(false);

const monitors = lazyRune(() => invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, monitors.setByPayload);

const widgetSize = lazyRune(() => Widget.self.webview.outerSize());
await Widget.self.webview.onResized(widgetSize.setByPayload);

await Promise.all([monitors.init(), widgetSize.init()]);

const primaryMonitor = $derived.by(() => {
  return monitors.value.find((m) => m.isPrimary) || monitors.value[0];
});

$effect.root(() => {
  $effect(() => {
    const monitor = primaryMonitor;

    if (!monitor || !showing) {
      return;
    }

    const placement = ConfigState.config.placement;
    const padding = ConfigState.config.margin * window.devicePixelRatio;

    const monitorWidth = monitor.rect.right - monitor.rect.left;
    const monitorHeight = monitor.rect.bottom - monitor.rect.top;

    const monitorCenterX = monitor.rect.left + monitorWidth / 2;
    const monitorCenterY = monitor.rect.top + monitorHeight / 2;

    const { width, height } = widgetSize.value;

    let x: number, y: number;
    if (placement === "left") {
      x = monitor.rect.left + padding;
      y = Math.round(monitorCenterY - height / 2);
    } else if (placement === "top") {
      x = Math.round(monitorCenterX - width / 2);
      y = monitor.rect.top + padding;
    } else if (placement === "right") {
      x = monitor.rect.right - width - padding;
      y = Math.round(monitorCenterY - height / 2);
    } else {
      x = Math.round(monitorCenterX - width / 2);
      y = monitor.rect.bottom - height - padding;
    }

    Widget.self.webview
      .setPosition(new PhysicalPosition(Math.round(x), Math.round(y)))
      .then(() => Widget.self.show());
  });
});

export function setShowing(value: boolean) {
  showing = value;

  if (!value) {
    Widget.self.hide();
  }
}

export const Monitors = {
  get all() {
    return monitors.value;
  },
  get primary() {
    return primaryMonitor;
  },
};
