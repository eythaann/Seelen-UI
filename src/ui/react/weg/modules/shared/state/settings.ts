import { lazySignal } from "libs/ui/react/utils/LazySignal";
import { Settings } from "@seelen-ui/lib";
import { Alignment } from "@seelen-ui/lib/types";
import { computed, effect } from "@preact/signals";
import i18n from "../../../i18n";
import { $current_monitor } from "./system";
import { toPhysicalPixels } from "libs/ui/react/utils";
import { SeelenWegSide } from "node_modules/@seelen-ui/lib/esm/gen/types/SeelenWegSide";

export const $settings = lazySignal(async () => {
  const settings = await Settings.getAsync();
  return {
    ...settings.byWidget["@seelen/weg"],
    language: settings.language,
    devTools: settings.devTools,
  };
});
Settings.onChange((settings) => {
  $settings.value = {
    ...settings.byWidget["@seelen/weg"],
    language: settings.language,
    devTools: settings.devTools,
  };
});
await $settings.init();

export const isHorizontalDock = computed(
  () =>
    $settings.value.position === SeelenWegSide.Top ||
    $settings.value.position === SeelenWegSide.Bottom,
);

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

export function getDockContextMenuAlignment(position: SeelenWegSide): { alignX: Alignment; alignY: Alignment } {
  switch (position) {
    case SeelenWegSide.Bottom:
      return { alignX: Alignment.Center, alignY: Alignment.End };
    case SeelenWegSide.Top:
      return { alignX: Alignment.Center, alignY: Alignment.Start };
    case SeelenWegSide.Left:
      return { alignX: Alignment.Start, alignY: Alignment.Center };
    case SeelenWegSide.Right:
      return { alignX: Alignment.End, alignY: Alignment.Center };
  }
}

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
