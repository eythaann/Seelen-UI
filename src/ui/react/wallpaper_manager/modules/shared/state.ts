import { computed, signal } from "@preact/signals";
import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, UIColors, WallpaperList } from "@seelen-ui/lib";
import { debounce } from "lodash";

const initial = await Settings.getAsync();

export const $settings = signal({
  ...initial.wall,
  byWallpaper: initial.byWallpaper,
  byMonitor: initial.monitorsV3,
});
Settings.onChange(
  (settings) => ($settings.value = {
    ...settings.wall,
    byWallpaper: settings.byWallpaper,
    byMonitor: settings.monitorsV3,
  }),
);

(await UIColors.getAsync()).setAsCssVariables();
UIColors.onChange((colors) => colors.setAsCssVariables());

export const $focused = signal(await invoke(SeelenCommand.GetFocusedApp));
subscribe(SeelenEvent.GlobalFocusChanged, (e) => {
  $focused.value = e.payload;
});

export const $idle = signal(false);
const setAsIdle = debounce(() => {
  $idle.value = true;
}, 1000 * 60 * 3); // 3 min
subscribe(SeelenEvent.GlobalMouseMove, () => {
  // avoid state change on every mouse move
  if ($idle.value) {
    $idle.value = false;
  }
  setAsIdle();
});

export const $muted = computed(() => {
  return $focused.value.class !== "Progman";
});

export const $paused = computed(() => {
  return (
    $idle.value ||
    ($focused.value.isFullscreened &&
      !$focused.value.exe?.toLowerCase().endsWith("explorer.exe")) ||
    $performance_mode.value !== "Disabled"
  );
});

export const $monitors = signal(await invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, ({ payload }) => {
  $monitors.value = payload;
});

export const $wallpapers = signal((await WallpaperList.getAsync()).asArray());
WallpaperList.onChange((wallpapers) => ($wallpapers.value = wallpapers.asArray()));

export const $performance_mode = signal(await invoke(SeelenCommand.StateGetPerformanceMode));
subscribe(SeelenEvent.StatePerformanceModeChanged, (e) => ($performance_mode.value = e.payload));
