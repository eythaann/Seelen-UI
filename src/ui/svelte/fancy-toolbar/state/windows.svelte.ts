import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { FancyToolbarSide, type UserAppWindowColors } from "@seelen-ui/lib/types";
import { lazyRune } from "libs/ui/svelte/utils";
import { settingsState, widgetRect } from "./settings.svelte.ts";

const widget = Widget.getCurrent();

export const interactables = lazyRune(() => invoke(SeelenCommand.GetUserAppWindows));
subscribe(SeelenEvent.UserAppWindowsChanged, interactables.setByPayload);
await interactables.init();

export const windowsColors = lazyRune<Record<number, UserAppWindowColors>>(
  () => invoke(SeelenCommand.GetUserAppWindowsColors),
);
subscribe(SeelenEvent.UserAppWindowsColorsChanged, windowsColors.setByPayload);
await windowsColors.init();

export const focused = lazyRune(() => invoke(SeelenCommand.GetFocusedApp));
subscribe(SeelenEvent.GlobalFocusChanged, focused.setByPayload);
await focused.init();

export const widgetStatuses = lazyRune(() => invoke(SeelenCommand.DebugGetWidgetsStatuses));
subscribe(SeelenEvent.WidgetDebugInfoChanged, widgetStatuses.setByPayload);
await widgetStatuses.init();

const _topInteractableWindow = $derived(
  interactables.value
    .toSorted((a, b) => b.lastForegroundAt - a.lastForegroundAt)
    .find((w) => w.monitor === widget.decoded.monitorId && !w.isIconic),
);

const _thereIsMaximizedOnBg = $derived(
  interactables.value.some(
    (w) => !w.isIconic && w.isZoomed && w.monitor === widget.decoded.monitorId,
  ),
);

const _isTbOverlapped = $derived.by(() => {
  const f = focused.value;
  const by = f?.monitor === widget.decoded.monitorId ? f : null;
  const ints = interactables.value;

  if (!by || !by.rect) return false;
  if (!ints.some((w) => w.hwnd === by.hwnd)) return false;

  const a = widgetRect.value;
  const b = by.rect;

  if (a.right <= b.left || a.left >= b.right || a.bottom <= b.top || a.top >= b.bottom) {
    return false;
  }

  return true;
});

const _currentMonitorMaximizedColors = $derived.by((): UserAppWindowColors | null => {
  const monitorId = widget.decoded.monitorId;
  const maximized = interactables.value.find(
    (w) => !w.isIconic && w.isZoomed && w.monitor === monitorId,
  );
  if (!maximized) return null;
  return windowsColors.value[maximized.hwnd] ?? null;
});

class WindowsState {
  get topInteractableWindow() {
    return _topInteractableWindow;
  }

  get thereIsMaximizedOnBg() {
    return _thereIsMaximizedOnBg;
  }

  get isTbOverlapped() {
    return _isTbOverlapped;
  }
}

export const windowsState = new WindowsState();

$effect.root(() => {
  $effect(() => {
    const colors = _currentMonitorMaximizedColors;
    const root = document.documentElement;

    root.dataset.thereIsMaximizedOnBg = `${!!colors}`;

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

    if (settingsState.position === FancyToolbarSide.Top) {
      root.style.setProperty("--window-gradient", toGradient(colors.top));
    } else {
      root.style.setProperty("--window-gradient", toGradient(colors.bottom));
    }
  });
});
