import { invoke, RuntimeStyleSheet, SeelenCommand, Widget } from "@seelen-ui/lib";
import { Alignment, FancyToolbarSide, HideMode, SeelenWegSide } from "@seelen-ui/lib/types";
import { isTouchPrimary } from "libs/ui/svelte/utils";
import { locale } from "../i18n/index.ts";
import { declareDocumentAsLayeredHitbox } from "libs/ui/react/utils/layered.ts";
import { systemState } from "./system.svelte.ts";
import { settings as _settings } from "./getters.svelte.ts";
import { dateState } from "libs/ui/svelte/runes/date.svelte.ts";

let isWidgetReady = $state(false);
const settings = $derived(_settings.value.byWidget["@seelen/weg"]);

const workArea = $derived.by(() => {
  const workArea = systemState.currentMonitor.rect;
  const tbConfig = _settings.value.byWidget["@seelen/fancy-toolbar"];
  const tbMonitorConfig = (_settings.value.monitorsV3[systemState.currentMonitor.id] as any)
    ?.byWidget?.["@seelen/fancy-toolbar"] || { enabled: true };

  if (!tbConfig?.enabled || !tbMonitorConfig?.enabled) {
    return workArea;
  }

  const tbSize = Math.round(
    (tbConfig.itemSize + tbConfig.padding * 2 + tbConfig.margin * 2) *
      systemState.currentMonitor.scaleFactor,
  );

  switch (tbConfig.position) {
    case FancyToolbarSide.Top:
      return { ...workArea, top: workArea.top + tbSize };
    case FancyToolbarSide.Bottom:
      return { ...workArea, bottom: workArea.bottom - tbSize };
  }

  return workArea;
});

const widgetRect = $derived.by(() => {
  const wa = workArea;
  const hitboxRect = { ...workArea };
  const webviewRect = { ...workArea };

  const size = Math.round(
    (settings.size + settings.padding * 2 + settings.margin * 2) *
      systemState.currentMonitor.scaleFactor,
  );

  switch (settings.position) {
    case SeelenWegSide.Left:
      hitboxRect.right = hitboxRect.left + size;
      webviewRect.right = isTouchPrimary.value ? hitboxRect.right : wa.right - Math.round((wa.right - wa.left) / 2);
      break;
    case SeelenWegSide.Right:
      hitboxRect.left = hitboxRect.right - size;
      webviewRect.left = isTouchPrimary.value ? hitboxRect.left : wa.left + Math.round((wa.right - wa.left) / 2);
      break;
    case SeelenWegSide.Top:
      hitboxRect.bottom = hitboxRect.top + size;
      webviewRect.bottom = isTouchPrimary.value ? hitboxRect.bottom : wa.top + Math.round((wa.bottom - wa.top) / 2);
      break;
    case SeelenWegSide.Bottom:
      hitboxRect.top = hitboxRect.bottom - size;
      webviewRect.top = isTouchPrimary.value ? hitboxRect.top : wa.bottom - Math.round((wa.bottom - wa.top) / 2);
      break;
  }

  return { hitboxRect, webviewRect };
});

$effect.root(() => {
  $effect(() => {
    locale.set(_settings.value.language);
    dateState.setLang(_settings.value.language);
    dateState.setFormat(_settings.value.dateFormat);
  });
});

export const fullSettings = {
  get value() {
    return _settings.value;
  },
};

class SettingsState {
  popupAlignX = $derived.by(() => {
    switch (settings.position) {
      case SeelenWegSide.Left:
        return Alignment.Start;
      case SeelenWegSide.Right:
        return Alignment.End;
      default:
        return Alignment.Center;
    }
  });

  popupAlignY = $derived.by(() => {
    switch (settings.position) {
      case SeelenWegSide.Bottom:
        return Alignment.End;
      case SeelenWegSide.Top:
        return Alignment.Start;
      default:
        return Alignment.Center;
    }
  });

  tooltipOrigin = $derived.by(() => {
    const origin: { x: number | null; y: number | null } = { x: null, y: null };
    const rect = settingsState.widgetRect.hitboxRect;
    switch (settingsState.position) {
      case SeelenWegSide.Left:
        origin.x = rect.right;
        break;
      case SeelenWegSide.Right:
        origin.x = rect.left;
        break;
      case SeelenWegSide.Top:
        origin.y = rect.bottom;
        break;
      case SeelenWegSide.Bottom:
        origin.y = rect.top;
        break;
    }
    return origin;
  });

  get isReady() {
    return isWidgetReady;
  }

  set isReady(v: boolean) {
    isWidgetReady = v;
  }

  get all() {
    return _settings.value;
  }

  get allByWidget() {
    return _settings.value.byWidget;
  }

  get value() {
    return settings;
  }

  get position(): SeelenWegSide {
    return settings.position;
  }

  get hideMode(): HideMode {
    return settings.hideMode;
  }

  get delayToHide(): number {
    return settings.delayToHide;
  }

  get delayToShow(): number {
    return settings.delayToShow;
  }

  get widgetRect() {
    return widgetRect;
  }
}

export const settingsState = new SettingsState();

export function isHorizontalDock(): boolean {
  const pos = settings.position;
  return pos === SeelenWegSide.Top || pos === SeelenWegSide.Bottom;
}

async function updateWidgetPosition() {
  const { hitboxRect, webviewRect } = widgetRect;
  const isTouch = isTouchPrimary.value;
  const hideMode = settings.hideMode;
  const position = settings.position;
  const isReady = settingsState.isReady;

  await Widget.self.setPosition(webviewRect);

  if (!isReady) {
    return;
  }

  if (hideMode === HideMode.Never || isTouch) {
    await invoke(SeelenCommand.RegisterAppBar, {
      rect: hitboxRect,
      edge: position as any,
    });
  } else {
    await invoke(SeelenCommand.UnregisterAppBar);
  }
}

Widget.self.window.onMoved(({ payload }) => {
  const rect = widgetRect.webviewRect;
  if (payload.x !== rect.left || payload.y !== rect.top) {
    Widget.self.setPosition(rect);
  }
});

$effect.root(() => {
  $effect(() => {
    const { size, padding, margin, spaceBetweenItems } = settings;
    const sheet = new RuntimeStyleSheet("@config/weg");
    sheet.addVariable("--config-margin", `${margin}px`);
    sheet.addVariable("--config-padding", `${padding}px`);
    sheet.addVariable("--config-item-size", `${size}px`);
    sheet.addVariable("--config-space-between-items", `${spaceBetweenItems}px`);
    sheet.applyToDocument();
  });

  $effect(() => {
    updateWidgetPosition();
  });

  $effect(() => {
    if (isTouchPrimary.value) return;

    let unlisten: (() => void) | null = null;
    declareDocumentAsLayeredHitbox({
      getPhysicalRect: () => {
        const r = widgetRect.webviewRect;
        return { x: r.left, y: r.top, width: r.right - r.left, height: r.bottom - r.top };
      },
    }).then((unlistenFn) => {
      unlisten = unlistenFn;
    });

    return () => {
      unlisten?.();
    };
  });
});
