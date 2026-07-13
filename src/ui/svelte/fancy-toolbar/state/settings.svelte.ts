import { invoke, RuntimeStyleSheet, SeelenCommand, Widget } from "@seelen-ui/lib";
import { type AppBarEdge, FancyToolbarSide, HideMode } from "@seelen-ui/lib/types";
import { isTouchPrimary } from "libs/ui/svelte/utils/signals.svelte.ts";
import { locale } from "../i18n/index.ts";
import { declareDocumentAsLayeredHitbox } from "libs/ui/react/utils/layered.ts";
import { systemState } from "./system.svelte.ts";
import { settings as _settings } from "./getters.svelte.ts";
import { dateState } from "libs/ui/svelte/runes/date.svelte.ts";

$effect.root(() => {
  $effect(() => {
    locale.set(_settings.value.language || "en");
    dateState.setLang(_settings.value.language);
    dateState.setFormat(_settings.value.dateFormat);
  });
});

let isWidgetReady = $state(false);

export const settingsState = {
  get isReady() {
    return isWidgetReady;
  },
  set isReady(v: boolean) {
    isWidgetReady = v;
  },

  get value() {
    return _settings.value.byWidget["@seelen/fancy-toolbar"];
  },

  get allByWidget() {
    return _settings.value.byWidget;
  },

  get itemSize(): number {
    return this.value.itemSize;
  },
  get margin(): number {
    return this.value.margin;
  },
  get padding(): number {
    return this.value.padding;
  },
  get position(): FancyToolbarSide {
    return this.value.position;
  },
  get hideMode(): HideMode {
    return this.value.hideMode;
  },
  get delayToHide(): number {
    return this.value.delayToHide;
  },
  get delayToShow(): number {
    return this.value.delayToShow;
  },
};

export const widgetRect = {
  get value() {
    const { itemSize, margin, padding } = settingsState;
    const height = Math.round(
      (itemSize + padding * 2 + margin * 2) * systemState.currentMonitor.scaleFactor,
    );
    const rect = { ...systemState.currentMonitor.rect };

    if (settingsState.position === FancyToolbarSide.Top) {
      rect.bottom = systemState.currentMonitor.rect.top + height;
    } else if (settingsState.position === FancyToolbarSide.Bottom) {
      rect.top = systemState.currentMonitor.rect.bottom - height;
    }

    return rect;
  },
};

async function updateWidgetPosition() {
  const isTouch = isTouchPrimary.value;
  const rect = widgetRect.value;
  const hideMode = settingsState.hideMode;
  const position = settingsState.position;
  const isReady = settingsState.isReady;

  await Widget.self.setPosition(rect);

  if (!isReady) {
    return;
  }

  if (hideMode === HideMode.Never || isTouch) {
    await invoke(SeelenCommand.RegisterAppBar, {
      rect,
      edge: position as unknown as AppBarEdge,
    });
  } else {
    await invoke(SeelenCommand.UnregisterAppBar);
  }
}

Widget.self.window.onMoved(({ payload }) => {
  if (payload.x !== widgetRect.value.left || payload.y !== widgetRect.value.top) {
    Widget.self.setPosition(widgetRect.value);
  }
});

$effect.root(() => {
  $effect(() => {
    const { itemSize, margin, padding } = settingsState;
    const sheet = new RuntimeStyleSheet("@config/fancy-toolbar");
    sheet.addVariable("--config-item-size", `${itemSize}px`);
    sheet.addVariable("--config-margin", `${margin}px`);
    sheet.addVariable("--config-padding", `${padding}px`);
    sheet.addVariable("--config-height", `${itemSize + padding * 2 + margin * 2}px`);
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
        const r = widgetRect.value;
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
