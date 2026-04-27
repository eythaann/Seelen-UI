import { computed, effect, signal } from "@preact/signals";
import { invoke, SeelenCommand, Settings, Widget } from "@seelen-ui/lib";
import { type AppBarEdge, FancyToolbarSide, HideMode } from "@seelen-ui/lib/types";
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
  const height = (itemSize + padding * 2 + margin * 2) * $current_monitor.value.scaleFactor;
  const rect = { ...$current_monitor.value.rect };

  if ($settings.value.position === FancyToolbarSide.Top) {
    rect.bottom = $current_monitor.value.rect.top + height;
  } else if ($settings.value.position === FancyToolbarSide.Bottom) {
    rect.top = $current_monitor.value.rect.bottom - height;
  }

  return rect;
});

export const $initialPositionSet = signal(false);

async function updateWidgetPosition() {
  const rect = $widget_rect.value;
  const hideMode = $settings.value.hideMode;
  const position = $settings.value.position;

  await Widget.self.setPosition(rect);

  if (hideMode === HideMode.Never) {
    await invoke(SeelenCommand.RegisterAppBar, {
      rect,
      edge: position as unknown as AppBarEdge,
    });
  } else {
    await invoke(SeelenCommand.UnregisterAppBar);
  }

  $initialPositionSet.value = true;
}

// setting an app bar, can cause move of the widget, this is to ensure correct position after such move
Widget.self.window.onMoved(({ payload }) => {
  if (payload.x !== $widget_rect.value.left || payload.y !== $widget_rect.value.top) {
    Widget.self.setPosition($widget_rect.value);
  }
});

effect(() => {
  updateWidgetPosition();
});
