import { lazySignal } from "libs/ui/react/utils/LazySignal";
import { invoke, SeelenCommand, Settings, Widget } from "@seelen-ui/lib";
import { Alignment, FancyToolbarSide, HideMode } from "@seelen-ui/lib/types";
import { computed, effect } from "@preact/signals";
import i18n from "../../../i18n";
import { $current_monitor } from "./system";
import { SeelenWegSide } from "node_modules/@seelen-ui/lib/esm/gen/types/SeelenWegSide";

export const $full_settings = lazySignal(async () => await Settings.getAsync());
Settings.onChange((s) => ($full_settings.value = s));
await $full_settings.init();

export const $settings = computed(() => $full_settings.value.byWidget["@seelen/weg"]);

export const isHorizontalDock = computed(
  () =>
    $settings.value.position === SeelenWegSide.Top ||
    $settings.value.position === SeelenWegSide.Bottom,
);

effect(() => {
  i18n.changeLanguage($full_settings.value.language || "en");
});

effect(() => {
  const settings = $settings.value;

  // @deprecated on future a utility function to parse widget settings as variables will be used.
  const styles = document.documentElement.style;

  styles.setProperty("--config-margin", `${settings.margin}px`);
  styles.setProperty("--config-padding", `${settings.padding}px`);

  styles.setProperty("--config-item-size", `${settings.size}px`);
  styles.setProperty("--config-item-zoom-size", `${settings.zoomSize}px`);
  styles.setProperty("--config-space-between-items", `${settings.spaceBetweenItems}px`);
});

export function getDockContextMenuAlignment(position: SeelenWegSide): {
  alignX: Alignment;
  alignY: Alignment;
} {
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

const $work_area = computed(() => {
  const workArea = $current_monitor.value.rect;
  const tbConfig = $full_settings.value.byWidget["@seelen/fancy-toolbar"];
  const tbMonitorConfig = $full_settings.value.monitorsV3[$current_monitor.value.id]?.byWidget?.[
    "@seelen/fancy-toolbar" as any
  ] || { enabled: true };

  if (!tbConfig.enabled || !tbMonitorConfig.enabled) {
    return workArea;
  }

  const tbSize = (tbConfig.itemSize + tbConfig.padding * 2 + tbConfig.margin * 2) *
    $current_monitor.value.scaleFactor;

  switch (tbConfig.position) {
    case FancyToolbarSide.Top:
      return {
        ...workArea,
        top: workArea.top + tbSize,
      };
    case FancyToolbarSide.Bottom:
      return {
        ...workArea,
        bottom: workArea.bottom - tbSize,
      };
  }

  return workArea;
});

export const $widget_rect = computed(() => {
  const workArea = { ...$work_area.value };

  const hitboxRect = { ...$work_area.value };
  const webviewRect = { ...$work_area.value };

  const size = ($settings.value.size + $settings.value.padding * 2 + $settings.value.margin * 2) *
    $current_monitor.value.scaleFactor;

  switch ($settings.value.position) {
    case SeelenWegSide.Left:
      hitboxRect.right = hitboxRect.left + size;
      webviewRect.right = workArea.right - (workArea.right - workArea.left) / 2;
      break;
    case SeelenWegSide.Right:
      hitboxRect.left = hitboxRect.right - size;
      webviewRect.left = workArea.left + (workArea.right - workArea.left) / 2;
      break;
    case SeelenWegSide.Top:
      hitboxRect.bottom = hitboxRect.top + size;
      webviewRect.bottom = workArea.top + (workArea.bottom - workArea.top) / 2;
      break;
    case SeelenWegSide.Bottom:
      hitboxRect.top = hitboxRect.bottom - size;
      webviewRect.top = workArea.bottom - (workArea.bottom - workArea.top) / 2;
      break;
  }

  return { hitboxRect, webviewRect };
});

effect(() => {
  const { hitboxRect, webviewRect } = $widget_rect.value;

  Widget.self.setPosition(webviewRect);

  if ($settings.value.hideMode === HideMode.Never) {
    invoke(SeelenCommand.RegisterAppBar, {
      rect: hitboxRect,
      edge: $settings.value.position as any,
    });
  } else {
    invoke(SeelenCommand.UnregisterAppBar);
  }
});
