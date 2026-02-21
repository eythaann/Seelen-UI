import { computed, signal } from "@preact/signals";
import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe } from "@seelen-ui/lib";
import { lazySignal } from "libs/ui/react/utils/LazySignal";
import { debounce } from "lodash";

const initial = await Settings.getAsync();

export const $settings = signal({
  ...initial.byWidget["@seelen/wallpaper-manager"],
  byWallpaper: initial.byWallpaper,
  byMonitor: initial.monitorsV3,
});
Settings.onChange(
  (settings) => ($settings.value = {
    ...settings.byWidget["@seelen/wallpaper-manager"],
    byWallpaper: settings.byWallpaper,
    byMonitor: settings.monitorsV3,
  }),
);

export const $focused = lazySignal(() => invoke(SeelenCommand.GetFocusedApp));
subscribe(SeelenEvent.GlobalFocusChanged, $focused.setByPayload);

export const $monitors = lazySignal(() => invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, $monitors.setByPayload);

export const $virtualDesktops = lazySignal(() => invoke(SeelenCommand.StateGetVirtualDesktops));
subscribe(SeelenEvent.VirtualDesktopsChanged, $virtualDesktops.setByPayload);

export const $wallpapers = lazySignal(() => invoke(SeelenCommand.StateGetWallpapers));
subscribe(SeelenEvent.StateWallpapersChanged, $wallpapers.setByPayload);

export const $performance_mode = lazySignal(() => invoke(SeelenCommand.StateGetPerformanceMode));
subscribe(SeelenEvent.StatePerformanceModeChanged, $performance_mode.setByPayload);

await Promise.all([
  $focused.init(),
  $monitors.init(),
  $virtualDesktops.init(),
  $wallpapers.init(),
  $performance_mode.init(),
]);

export const $idle = signal(false);
const setAsIdle = debounce(
  () => {
    $idle.value = true;
  },
  1000 * 60 * 3,
); // 3 min
subscribe(SeelenEvent.GlobalMouseMove, () => {
  // avoid state change on every mouse move
  if ($idle.value) {
    $idle.value = false;
  }
  setAsIdle();
});

export const $muted = computed(() => {
  return !["Progman", "SysListView32"].includes($focused.value.class);
});

export const $paused = computed(() => {
  return (
    $idle.value ||
    ($focused.value.isFullscreened &&
      !$focused.value.exe?.toLowerCase().endsWith("explorer.exe")) ||
    $performance_mode.value !== "Disabled"
  );
});
