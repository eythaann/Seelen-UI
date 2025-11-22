import { lazySignal } from "@shared/LazySignal";
import { Settings } from "@seelen-ui/lib";
import { computed, effect } from "@preact/signals";
import i18n from "../../../i18n";
import { $current_monitor } from "./system";
import { toPhysicalPixels } from "@shared";
import { SeelenWegSide } from "node_modules/@seelen-ui/lib/esm/gen/types/SeelenWegSide";

export const $settings = lazySignal(async () => {
  const settings = await Settings.getAsync();
  return {
    ...settings.seelenweg,
    language: settings.language,
    devTools: settings.devTools,
  };
});
await Settings.onChange((settings) => {
  $settings.value = {
    ...settings.seelenweg,
    language: settings.language,
    devTools: settings.devTools,
  };
});
await $settings.init();

effect(() => {
  const settings = $settings.value;
  i18n.changeLanguage(settings.language || undefined);

  // @deprecated on future a utility function to parse widget settings as variables will be used.
  const styles = document.documentElement.style;

  styles.setProperty("--config-margin", `${settings.margin}px`);
  styles.setProperty("--config-padding", `${settings.padding}px`);

  styles.setProperty("--config-item-size", `${settings.size}px`);
  styles.setProperty("--config-item-zoom-size", `${settings.zoomSize}px`);
  styles.setProperty("--config-space-between-items", `${settings.spaceBetweenItems}px`);
});

export const $widget_rect = computed(() => {
  const rect = { ...$current_monitor.value.rect };
  const size = toPhysicalPixels(
    $settings.value.size + $settings.value.padding * 2 + $settings.value.margin * 2,
  );

  switch ($settings.value.position) {
    case SeelenWegSide.Left:
      rect.right = rect.left + size;
      break;
    case SeelenWegSide.Right:
      rect.left = rect.right - size;
      break;
    case SeelenWegSide.Top:
      rect.bottom = rect.top + size;
      break;
    case SeelenWegSide.Bottom:
      rect.top = rect.bottom - size;
      break;
  }

  return rect;
});
