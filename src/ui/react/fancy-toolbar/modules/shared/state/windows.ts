import { computed, effect, signal } from "@preact/signals";
import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { FancyToolbarSide, HideMode, type UserAppWindowColors } from "@seelen-ui/lib/types";
import { $settings, $widget_rect } from "./mod";
import { $mouse_at_edge } from "./system";
import { $is_this_webview_focused, $is_touch_primary } from "libs/ui/react/utils/signals";
import { lazySignal } from "libs/ui/react/utils/LazySignal";

const widget = Widget.getCurrent();

export const $interactables = lazySignal(() => invoke(SeelenCommand.GetUserAppWindows));
subscribe(SeelenEvent.UserAppWindowsChanged, $interactables.setByPayload);
await $interactables.init();

export const $thereIsMaximizedOnBg = computed(() => {
  return $interactables.value.some(
    (w) => !w.isIconic && w.isZoomed && w.monitor === widget.decoded.monitorId,
  );
});

export const $isSomeFullscreenOnMonitor = computed(() => {
  return $interactables.value.some(
    (w) => !w.isIconic && w.isFullscreen && w.monitor === widget.decoded.monitorId,
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

export const $hidden_by_autohide = signal(false);

effect(() => {
  const { delayToHide, delayToShow, hideMode, position } = $settings.value;

  let hidden = false;
  let flush = false;

  const isMouseOverEdge = $mouse_at_edge.value === position;

  switch (hideMode) {
    case HideMode.Never:
      hidden = false;
      flush = true;
      break;
    case HideMode.Always:
      hidden = !$is_touch_primary.value && !$is_this_webview_focused.value && !isMouseOverEdge;
      flush = $is_touch_primary.value;
      break;
    case HideMode.OnOverlap:
      hidden = !$is_touch_primary.value &&
        $is_tb_overlapped.value &&
        !$is_this_webview_focused.value &&
        !isMouseOverEdge;
      flush = $is_touch_primary.value;
      break;
  }

  let timeout: ReturnType<typeof setTimeout> | null = null;
  if (hidden) {
    timeout = setTimeout(() => ($hidden_by_autohide.value = true), delayToHide);
  } else {
    if (flush) {
      $hidden_by_autohide.value = false;
    } else {
      timeout = setTimeout(() => ($hidden_by_autohide.value = false), delayToShow);
    }
  }

  return () => {
    if (timeout) {
      clearTimeout(timeout);
    }
  };
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
