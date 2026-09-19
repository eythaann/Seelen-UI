import { Widget } from "@seelen-ui/lib";
import { FancyToolbarSide, type UserAppWindowColors } from "@seelen-ui/lib/types";
import { settingsState } from "./settings.svelte.ts";
import { focused, interactables, widgetStatuses, windowsColors } from "./getters.svelte.ts";

export { focused, interactables, widgetStatuses, windowsColors };

type WindowColor = UserAppWindowColors["top"][number];

function relativeLuminance({ r, g, b }: WindowColor): number {
  const toLinear = (channel: number) => {
    const normalized = channel / 255;
    return normalized <= 0.03928
      ? normalized / 12.92
      : ((normalized + 0.055) / 1.055) ** 2.4;
  };

  return 0.2126 * toLinear(r) + 0.7152 * toLinear(g) + 0.0722 * toLinear(b);
}

function averageLuminance(colors: WindowColor[]): number {
  if (!colors.length) return 0;
  return colors.reduce((total, color) => total + relativeLuminance(color), 0) / colors.length;
}

const widget = Widget.getCurrent();

const _topInteractableWindow = $derived(
  interactables.value
    .toSorted((a, b) => b.lastForegroundAt - a.lastForegroundAt)
    .find((w) => w.monitor === widget.decoded.monitorId && !w.isIconic),
);

const _isTbOverlapped = $derived.by(() => {
  // If foreground is not in interactable windows, return false directly, this handled start menu or desktop focus cases.
  const foreground = focused.value;
  if (!interactables.value.some((w) => w.hwnd === foreground.hwnd)) {
    return false;
  }

  // Check if any interactable window overlaps with the hitbox
  const a = settingsState.widgetRect;
  for (const app of interactables.value) {
    if (app.monitor !== widget.decoded.monitorId || app.isIconic || !app.rect) continue;
    const b = app.rect;

    if (!(a.right <= b.left || a.left >= b.right || a.bottom <= b.top || a.top >= b.bottom)) {
      return true;
    }
  }

  return false;
});

const _currentMonitorMaximizedWindow = $derived.by(() => {
  const monitorId = widget.decoded.monitorId;
  return interactables.value
    .toSorted((a, b) => b.lastForegroundAt - a.lastForegroundAt)
    .find((w) => !w.isIconic && w.isZoomed && w.monitor === monitorId);
});

const _currentMonitorMaximizedColors = $derived.by((): UserAppWindowColors | null => {
  const maximized = _currentMonitorMaximizedWindow;
  if (!maximized) return null;
  return windowsColors.value[maximized.hwnd] ?? null;
});

const _thereIsMaximizedOnBg = $derived(_currentMonitorMaximizedColors !== null);

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
      // The non-maximized theme uses a light glass surface by default.
      root.dataset.toolbarForeground = "dark";
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

    const stops = settingsState.position === FancyToolbarSide.Top ? colors.top : colors.bottom;
    root.dataset.toolbarForeground = averageLuminance(stops) >= 0.45 ? "dark" : "light";

    if (settingsState.position === FancyToolbarSide.Top) {
      root.style.setProperty("--window-gradient", toGradient(colors.top));
    } else {
      root.style.setProperty("--window-gradient", toGradient(colors.bottom));
    }
  });
});
