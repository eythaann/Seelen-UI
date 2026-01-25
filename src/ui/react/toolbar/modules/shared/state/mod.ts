import { computed, signal } from "@preact/signals";
import { Settings } from "@seelen-ui/lib";
import { FancyToolbarSide } from "@seelen-ui/lib/types";
import { toPhysicalPixels } from "libs/ui/react/utils";
import { $current_monitor } from "./system";

const initialSettings = await Settings.getAsync();
export const $settings = signal({
  ...initialSettings.byWidget["@seelen/fancy-toolbar"],
  language: initialSettings.language || "en",
  dateFormat: initialSettings.dateFormat,
  startOfWeek: initialSettings.startOfWeek,
});
Settings.onChange(
  (settings) => ($settings.value = {
    ...settings.byWidget["@seelen/fancy-toolbar"],
    language: settings.language || "en",
    dateFormat: settings.dateFormat,
    startOfWeek: settings.startOfWeek,
  }),
);

export const $widget_rect = computed(() => {
  const height = toPhysicalPixels($settings.value.height);
  const rect = { ...$current_monitor.value.rect };

  if ($settings.value.position === FancyToolbarSide.Top) {
    rect.bottom = $current_monitor.value.rect.top + height;
  } else if ($settings.value.position === FancyToolbarSide.Bottom) {
    rect.top = $current_monitor.value.rect.bottom - height;
  }

  return rect;
});
