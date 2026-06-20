import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { Wallpaper } from "@seelen-ui/lib/types";
import { locale } from "./i18n/index.ts";
import { debounce } from "lodash";
import { calculateMonitorCoverage } from "./utils/monitorCoverage.ts";
import {
  focused,
  interactables,
  monitors,
  performanceMode,
  players,
  settings as _settings,
  virtualDesktops,
  wallpapers,
} from "./getters.svelte.ts";
import WallpaperState from "libs/ui/svelte/components/Wallpaper/state.svelte.ts";

const settings = $derived.by(() => ({
  ..._settings.value.byWidget["@seelen/wallpaper-manager"],
  byWallpaper: _settings.value.byWallpaper,
  byMonitor: _settings.value.monitorsV3,
}));

$effect.root(() => {
  $effect(() => {
    locale.set(_settings.value.language || "en");
  });

  $effect(() => {
    WallpaperState.player = players.value.find((p) => p.default) || null;
  });
});

let idle = $state(false);
const setAsIdle = debounce(
  () => {
    idle = true;
  },
  1000 * 60 * 3,
); // 3 min
subscribe(SeelenEvent.GlobalMouseMove, () => {
  if (idle) idle = false;
  setAsIdle();
});

const muted = $derived(!["Progman", "SysListView32"].includes(focused.value.class));

const pausedMonitors = $derived.by(() => {
  if (performanceMode.value !== "Disabled" || idle) {
    return new Set(monitors.value.map((m) => m.id));
  }

  const paused = new Set<string>();
  const visibleWindows = interactables.value.filter((w) => !w.isIconic && w.rect != null);
  for (const monitor of monitors.value) {
    const windowRects = visibleWindows.filter((w) => w.monitor === monitor.id).map((w) => w.rect!);
    if (calculateMonitorCoverage(monitor.rect, windowRects) > settings.coveragePauseThreshold) {
      paused.add(monitor.id);
    }
  }
  return paused;
});

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
  isPaused(monitorId: string): boolean {
    return pausedMonitors.has(monitorId);
  }
  get players() {
    return players.value;
  }

  findWallpaper(wallpaperId: string | null | undefined): Wallpaper | undefined {
    if (!wallpaperId) return undefined;
    return this.wallpapers.find((w) => w.id === wallpaperId);
  }
}

$effect.root(() => {
  $effect(() => {
    monitors.value;
    // wait to desktop to be ready after monitor change
    setTimeout(() => {
      invoke(SeelenCommand.SetAsWallpaper);
    }, 2000);
  });
});

export const gState = new State();
