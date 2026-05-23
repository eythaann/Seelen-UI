import { lazySignal } from "libs/ui/react/utils/LazySignal";
import { declareDocumentAsLayeredHitbox } from "libs/ui/react/utils/layered.ts";
import { invoke, RuntimeStyleSheet, SeelenCommand, Settings, Widget } from "@seelen-ui/lib";
import { Alignment, FancyToolbarSide, HideMode } from "@seelen-ui/lib/types";
import { computed, effect, signal } from "@preact/signals";
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

  const sheet = new RuntimeStyleSheet("@config/weg");
  sheet.addVariable("--config-margin", `${settings.margin}px`);
  sheet.addVariable("--config-padding", `${settings.padding}px`);
  sheet.addVariable("--config-item-size", `${settings.size}px`);
  sheet.addVariable("--config-item-zoom-size", `${settings.zoomSize}px`);
  sheet.addVariable("--config-space-between-items", `${settings.spaceBetweenItems}px`);
  sheet.applyToDocument();
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

  const tbSize = Math.round(
    (tbConfig.itemSize + tbConfig.padding * 2 + tbConfig.margin * 2) *
      $current_monitor.value.scaleFactor,
  );

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

const _pointerQuery = window.matchMedia("(hover: hover) and (pointer: fine)");
export const $isTouchPrimary = signal(!_pointerQuery.matches);
_pointerQuery.addEventListener("change", (e) => {
  $isTouchPrimary.value = !e.matches;
});

effect(() => {
  if ($isTouchPrimary.value) {
    Widget.self.window.setIgnoreCursorEvents(false);
    return;
  }
  let unlisten: (() => void) | null = null;
  declareDocumentAsLayeredHitbox().then((fn) => {
    unlisten = fn;
  });
  return () => unlisten?.();
});

export const $widget_rect = computed(() => {
  const workArea = { ...$work_area.value };

  const hitboxRect = { ...$work_area.value };
  const webviewRect = { ...$work_area.value };

  const size = Math.round(
    ($settings.value.size + $settings.value.padding * 2 + $settings.value.margin * 2) *
      $current_monitor.value.scaleFactor,
  );

  switch ($settings.value.position) {
    case SeelenWegSide.Left:
      hitboxRect.right = hitboxRect.left + size;
      webviewRect.right = $isTouchPrimary.value
        ? hitboxRect.right
        : workArea.right - Math.round((workArea.right - workArea.left) / 2);
      break;
    case SeelenWegSide.Right:
      hitboxRect.left = hitboxRect.right - size;
      webviewRect.left = $isTouchPrimary.value
        ? hitboxRect.left
        : workArea.left + Math.round((workArea.right - workArea.left) / 2);
      break;
    case SeelenWegSide.Top:
      hitboxRect.bottom = hitboxRect.top + size;
      webviewRect.bottom = $isTouchPrimary.value
        ? hitboxRect.bottom
        : workArea.top + Math.round((workArea.bottom - workArea.top) / 2);
      break;
    case SeelenWegSide.Bottom:
      hitboxRect.top = hitboxRect.bottom - size;
      webviewRect.top = $isTouchPrimary.value
        ? hitboxRect.top
        : workArea.bottom - Math.round((workArea.bottom - workArea.top) / 2);
      break;
  }

  return { hitboxRect, webviewRect };
});

export const $initialPositionSet = signal(false);
async function updateWidgetPosition() {
  const { hitboxRect, webviewRect } = $widget_rect.value;
  const hideMode = $settings.value.hideMode;
  const position = $settings.value.position;

  await Widget.self.setPosition(webviewRect);

  if (hideMode === HideMode.Never) {
    await invoke(SeelenCommand.RegisterAppBar, {
      rect: hitboxRect,
      edge: position as any,
    });
  } else {
    await invoke(SeelenCommand.UnregisterAppBar);
  }

  $initialPositionSet.value = true;
}

// setting an app bar, can cause move of the widget, this is to ensure correct position after such move
Widget.self.window.onMoved(({ payload }) => {
  const rect = $widget_rect.value.webviewRect;
  if (payload.x !== rect.left || payload.y !== rect.top) {
    Widget.self.setPosition(rect);
  }
});

effect(() => {
  updateWidgetPosition();
});
