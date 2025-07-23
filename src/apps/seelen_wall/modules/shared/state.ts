import { signal } from '@preact/signals';
import {
  invoke,
  SeelenCommand,
  SeelenEvent,
  Settings,
  subscribe,
  UIColors,
  WallpaperList,
} from '@seelen-ui/lib';

const initial = await Settings.getAsync();

export const $settings = signal({
  ...initial.wall,
  byWallpaper: initial.byWallpaper,
  byMonitor: initial.monitorsV3,
});
Settings.onChange(
  (settings) =>
    ($settings.value = {
      ...settings.wall,
      byWallpaper: settings.byWallpaper,
      byMonitor: settings.monitorsV3,
    }),
);

(await UIColors.getAsync()).setAsCssVariables();
UIColors.onChange((colors) => colors.setAsCssVariables());

export const $paused = signal(false);
subscribe(SeelenEvent.WallStop, ({ payload }) => ($paused.value = payload));

export const $monitors = signal(await invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, ({ payload }) => {
  $monitors.value = payload;
});

export const $wallpapers = signal((await WallpaperList.getAsync()).asArray());
WallpaperList.onChange((wallpapers) => ($wallpapers.value = wallpapers.asArray()));
