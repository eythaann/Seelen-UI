import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe } from "@seelen-ui/lib";
import { lazyRune } from "libs/ui/svelte/utils";
import { locale } from "./i18n/index.ts";
import { debounce } from "lodash";

let _settings = $state(await Settings.getAsync());
Settings.onChange((s) => (_settings = s));

const settings = $derived.by(() => ({
  ..._settings.byWidget["@seelen/wallpaper-manager"],
  byWallpaper: _settings.byWallpaper,
  byMonitor: _settings.monitorsV3,
}));

$effect.root(() => {
  $effect(() => {
    locale.set(_settings.language || "en");
  });
});

const focused = lazyRune(() => invoke(SeelenCommand.GetFocusedApp));
subscribe(SeelenEvent.GlobalFocusChanged, focused.setByPayload);

const monitors = lazyRune(() => invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, monitors.setByPayload);

const virtualDesktops = lazyRune(() => invoke(SeelenCommand.StateGetVirtualDesktops));
subscribe(SeelenEvent.VirtualDesktopsChanged, virtualDesktops.setByPayload);

const wallpapers = lazyRune(() => invoke(SeelenCommand.StateGetWallpapers));
subscribe(SeelenEvent.StateWallpapersChanged, wallpapers.setByPayload);

const performanceMode = lazyRune(() => invoke(SeelenCommand.StateGetPerformanceMode));
subscribe(SeelenEvent.StatePerformanceModeChanged, performanceMode.setByPayload);

await Promise.all([
  focused.init(),
  monitors.init(),
  virtualDesktops.init(),
  wallpapers.init(),
  performanceMode.init(),
]);

let idle = $state(false);
const setAsIdle = debounce(() => {
  idle = true;
}, 1000 * 60 * 3); // 3 min
subscribe(SeelenEvent.GlobalMouseMove, () => {
  if (idle) idle = false;
  setAsIdle();
});

const muted = $derived(!["Progman", "SysListView32"].includes(focused.value.class));

const paused = $derived(
  idle ||
    (focused.value.isFullscreened &&
      !focused.value.exe?.toLowerCase().endsWith("explorer.exe")) ||
    performanceMode.value !== "Disabled",
);

const desktopRect = $derived.by(() => {
  let rect = { top: 0, left: 0, right: 0, bottom: 0 };
  for (const monitor of monitors.value) {
    rect.left = Math.min(rect.left, monitor.rect.left);
    rect.top = Math.min(rect.top, monitor.rect.top);
    rect.right = Math.max(rect.right, monitor.rect.right);
    rect.bottom = Math.max(rect.bottom, monitor.rect.bottom);
  }
  return rect;
});

const relativeMonitors = $derived.by(() => {
  return monitors.value.map((monitor) => ({
    ...monitor,
    rect: {
      ...monitor.rect,
      left: monitor.rect.left - desktopRect.left,
      top: monitor.rect.top - desktopRect.top,
      right: monitor.rect.right - desktopRect.left,
      bottom: monitor.rect.bottom - desktopRect.top,
    },
  }));
});

class State {
  get settings() {
    return settings;
  }
  get monitors() {
    return monitors.value;
  }
  get relativeMonitors() {
    return relativeMonitors;
  }
  get virtualDesktops() {
    return virtualDesktops.value;
  }
  get wallpapers() {
    return wallpapers.value;
  }
  get performanceMode() {
    return performanceMode.value;
  }
  get muted() {
    return muted;
  }
  get paused() {
    return paused;
  }

  findWallpaper(wallpaperId: string | null | undefined) {
    if (!wallpaperId) return undefined;
    return this.wallpapers.find((w) => w.id === wallpaperId);
  }
}

export const gState = new State();
