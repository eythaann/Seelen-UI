import { computed, effect } from "@preact/signals";
import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { FancyToolbarSide, type UserAppWindowColors } from "@seelen-ui/lib/types";
import { $settings, $widget_rect } from "./settings";
import { lazySignal } from "libs/ui/react/utils/LazySignal";

const widget = Widget.getCurrent();

export const $interactables = lazySignal(() => invoke(SeelenCommand.GetUserAppWindows));
subscribe(SeelenEvent.UserAppWindowsChanged, $interactables.setByPayload);
await $interactables.init();

export const $top_interactable_window = computed(() =>
  $interactables.value
    .toSorted((a, b) => b.lastForegroundAt - a.lastForegroundAt)
    .find((w) => w.monitor === widget.decoded.monitorId && !w.isIconic)
);

export const $thereIsMaximizedOnBg = computed(() => {
  return $interactables.value.some(
    (w) => !w.isIconic && w.isZoomed && w.monitor === widget.decoded.monitorId,
  );
});

export const $windowsColors = lazySignal<Record<number, UserAppWindowColors>>(() =>
  invoke(SeelenCommand.GetUserAppWindowsColors)
);
subscribe(SeelenEvent.UserAppWindowsColorsChanged, $windowsColors.setByPayload);
await $windowsColors.init();

export const $currentMonitorMaximizedColors = computed<UserAppWindowColors | null>(() => {
  const monitorId = widget.decoded.monitorId;
  const maximized = $interactables.value.find(
    (w) => !w.isIconic && w.isZoomed && w.monitor === monitorId,
  );
  if (!maximized) return null;
  return $windowsColors.value[maximized.hwnd] ?? null;
});

export const $focused = lazySignal(() => invoke(SeelenCommand.GetFocusedApp));
subscribe(SeelenEvent.GlobalFocusChanged, $focused.setByPayload);
await $focused.init();

export const $widget_statuses = lazySignal(() => invoke(SeelenCommand.DebugGetWidgetsStatuses));
subscribe(SeelenEvent.WidgetDebugInfoChanged, $widget_statuses.setByPayload);
await $widget_statuses.init();

export const $is_tb_overlapped = computed(() => {
  const focused = $focused.value;
  const by = focused?.monitor === widget.decoded.monitorId ? focused : null;
  const interactables = $interactables.value;

  if (!by || !by.rect) {
    return false;
  }

  if (!interactables.some((w) => w.hwnd === by.hwnd)) {
    return false;
  }

  const a = $widget_rect.value;
  const b = by.rect;

  // The edge pixel overlapping do not matters. This resolves the shared pixel in between the monitors,
  // hereby a fullscreened app shared pixel collision does not hide other monitor windows.
  if (a.right <= b.left || a.left >= b.right || a.bottom <= b.top || a.top >= b.bottom) {
    return false;
  }

  return true;
});

effect(() => {
  const colors = $currentMonitorMaximizedColors.value;
  const root = document.documentElement;

  if (!colors) {
    root.style.removeProperty("--window-gradient");
    return;
  }

  const toRgba = ({ r, g, b, a }: { r: number; g: number; b: number; a: number }) =>
    `rgba(${r},${g},${b},${(a / 255).toFixed(3)})`;

  const toGradient = (stops: typeof colors.top) =>
    `linear-gradient(to right,${
      stops
        .map((c, i) => `${toRgba(c)} ${((i / (stops.length - 1)) * 100).toFixed(1)}%`)
        .join(",")
    })`;

  if ($settings.value.position === FancyToolbarSide.Top) {
    root.style.setProperty("--window-gradient", toGradient(colors.top));
  } else {
    root.style.setProperty("--window-gradient", toGradient(colors.bottom));
  }
});
