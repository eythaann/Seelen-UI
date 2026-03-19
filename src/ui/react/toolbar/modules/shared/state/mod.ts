import { computed, effect, signal } from "@preact/signals";
import { Settings } from "@seelen-ui/lib";
import { FancyToolbarSide } from "@seelen-ui/lib/types";
import { toPhysicalPixels } from "libs/ui/react/utils";
import { $current_monitor } from "./system";
import i18n from "../../../i18n";

const initialSettings = await Settings.getAsync();
export const $settings = signal({
  ...initialSettings.byWidget["@seelen/fancy-toolbar"],
  language: initialSettings.language || "en",
  dateFormat: initialSettings.dateFormat,
  startOfWeek: initialSettings.startOfWeek,
});

export const $allByWidget = signal(initialSettings.byWidget);
Settings.onChange((settings) => {
  $settings.value = {
    ...settings.byWidget["@seelen/fancy-toolbar"],
    language: settings.language || "en",
    dateFormat: settings.dateFormat,
    startOfWeek: settings.startOfWeek,
  };
  $allByWidget.value = settings.byWidget;
});

effect(() => {
  const { itemSize, margin, padding } = $settings.value;
  i18n.changeLanguage($settings.value.language || undefined);

  const styles = document.documentElement.style;
  styles.setProperty("--config-item-size", `${itemSize}px`);
  styles.setProperty("--config-margin", `${margin}px`);
  styles.setProperty("--config-padding", `${padding}px`);
  styles.setProperty("--config-height", `${itemSize + padding * 2 + margin * 2}px`);
});

export const $widget_rect = computed(() => {
  const { itemSize, margin, padding } = $settings.value;
  const height = toPhysicalPixels(itemSize + padding * 2 + margin * 2);
  const rect = { ...$current_monitor.value.rect };

  if ($settings.value.position === FancyToolbarSide.Top) {
    rect.bottom = $current_monitor.value.rect.top + height;
  } else if ($settings.value.position === FancyToolbarSide.Bottom) {
    rect.top = $current_monitor.value.rect.bottom - height;
  }

  return rect;
});
