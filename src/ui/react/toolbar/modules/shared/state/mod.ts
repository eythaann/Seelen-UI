import { computed, signal } from "@preact/signals";
import { Settings } from "@seelen-ui/lib";
import { type FancyToolbarSettings, FancyToolbarSide } from "@seelen-ui/lib/types";
import { toPhysicalPixels } from "@shared";
import { $current_monitor } from "./system";

const initialSettings = await Settings.getAsync();
export const $settings = signal<FancyToolbarSettings & Pick<Settings, "dateFormat">>({
  ...initialSettings.fancyToolbar,
  dateFormat: initialSettings.dateFormat,
});
Settings.onChange(
  (settings) => ($settings.value = {
    ...settings.fancyToolbar,
    dateFormat: settings.dateFormat,
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
