import { invoke, RuntimeStyleSheet, SeelenCommand, Widget } from "@seelen-ui/lib";
import { Alignment, FancyToolbarSide, HideMode, SeelenWegSide } from "@seelen-ui/lib/types";
import { isTouchPrimary } from "libs/ui/svelte/utils";
import { locale } from "../i18n/index.ts";
import { declareDocumentAsLayeredHitbox } from "libs/ui/react/utils/layered.ts";
import { systemState } from "./system.svelte.ts";
import { settings as _settings } from "./getters.svelte.ts";

$effect.root(() => {
  $effect(() => {
    locale.set(_settings.value.language || "en");
  });
});

export const fullSettings = {
  get value() {
    return _settings.value;
  },
};

let isWidgetReady = $state(false);

export const settingsState = {
  get isReady() {
    return isWidgetReady;
  },
  set isReady(v: boolean) {
    isWidgetReady = v;
  },
  get value() {
    return _settings.value.byWidget["@seelen/weg"] as any;
  },
  get position(): SeelenWegSide {
    return (this.value?.position ?? SeelenWegSide.Bottom) as SeelenWegSide;
  },
  get hideMode(): HideMode {
    return (this.value?.hideMode ?? HideMode.Never) as HideMode;
  },
  get delayToHide(): number {
    return (this.value?.delayToHide ?? 800) as number;
  },
  get delayToShow(): number {
    return (this.value?.delayToShow ?? 300) as number;
  },
  get size(): number {
    return (this.value?.size ?? 40) as number;
  },
  get padding(): number {
    return (this.value?.padding ?? 4) as number;
  },
  get margin(): number {
    return (this.value?.margin ?? 0) as number;
  },
};

export function isHorizontalDock(): boolean {
  const pos = settingsState.position;
  return pos === SeelenWegSide.Top || pos === SeelenWegSide.Bottom;
}

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

const workArea = {
  get value() {
    const workArea = systemState.currentMonitor.rect;
    const tbConfig = _settings.value.byWidget["@seelen/fancy-toolbar"] as any;
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
  },
};

export const widgetRect = {
  get value() {
    const wa = { ...workArea.value };
    const hitboxRect = { ...workArea.value };
    const webviewRect = { ...workArea.value };

    const size = Math.round(
      (settingsState.size + settingsState.padding * 2 + settingsState.margin * 2) *
        systemState.currentMonitor.scaleFactor,
    );

    switch (settingsState.position) {
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
  },
};

async function updateWidgetPosition() {
  const { hitboxRect, webviewRect } = widgetRect.value;
  const isTouch = isTouchPrimary.value;
  const hideMode = settingsState.hideMode;
  const position = settingsState.position;
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
  const rect = widgetRect.value.webviewRect;
  if (payload.x !== rect.left || payload.y !== rect.top) {
    Widget.self.setPosition(rect);
  }
});

$effect.root(() => {
  $effect(() => {
    const { size, padding, margin, spaceBetweenItems, zoomSize } = settingsState.value || {};
    const sheet = new RuntimeStyleSheet("@config/weg");
    sheet.addVariable("--config-margin", `${margin ?? 0}px`);
    sheet.addVariable("--config-padding", `${padding ?? 0}px`);
    sheet.addVariable("--config-item-size", `${size ?? 40}px`);
    sheet.addVariable("--config-item-zoom-size", `${zoomSize ?? 40}px`);
    sheet.addVariable("--config-space-between-items", `${spaceBetweenItems ?? 0}px`);
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
        const r = widgetRect.value.webviewRect;
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
