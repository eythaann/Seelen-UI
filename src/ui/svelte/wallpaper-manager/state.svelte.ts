import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { Wallpaper } from "@seelen-ui/lib/types";
import { locale } from "./i18n/index.ts";
import { debounce } from "lodash";
import { untrack } from "svelte";
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
    locale.set(_settings.value.language);
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

// how long a monitor must stay uncovered before actually unpausing, to avoid
// pause/unpause flicker during fast window rearrangements (e.g. workspace switch)
const UNPAUSE_DEBOUNCE_MS = 400;

const desiredPausedMonitors = $derived.by(() => {
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

let pausedMonitors = $state<Set<string>>(new Set());
const unpauseTimers = new Map<string, ReturnType<typeof setTimeout>>();

$effect.root(() => {
  $effect(() => {
    const desired = desiredPausedMonitors;

    untrack(() => {
      // covered monitors pause immediately, and cancel any pending unpause
      for (const monitorId of desired) {
        const timer = unpauseTimers.get(monitorId);
        if (timer) {
          clearTimeout(timer);
          unpauseTimers.delete(monitorId);
        }
        if (!pausedMonitors.has(monitorId)) {
          pausedMonitors = new Set(pausedMonitors).add(monitorId);
        }
      }

      // monitors no longer covered wait a bit before actually unpausing
      for (const monitorId of pausedMonitors) {
        if (desired.has(monitorId) || unpauseTimers.has(monitorId)) continue;
        unpauseTimers.set(
          monitorId,
          setTimeout(() => {
            unpauseTimers.delete(monitorId);
            pausedMonitors = new Set([...pausedMonitors].filter((id) => id !== monitorId));
          }, UNPAUSE_DEBOUNCE_MS),
        );
      }
    });
  });
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
    relativeMonitors;
    invoke(SeelenCommand.SetAsWallpaper);
  });
});

export const gState = new State();
