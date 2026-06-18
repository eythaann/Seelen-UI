import { invoke, RuntimeStyleSheet, SeelenCommand, Settings, Widget } from "@seelen-ui/lib";
import { type AppBarEdge, FancyToolbarSide, HideMode } from "@seelen-ui/lib/types";
import { lazyRune } from "libs/ui/svelte/utils";
import { isTouchPrimary } from "libs/ui/svelte/utils/signals.svelte.ts";
import { locale } from "../i18n/index.ts";
import { declareDocumentAsLayeredHitbox } from "libs/ui/react/utils/layered.ts";
import { systemState } from "./system.svelte.ts";

const _settings = lazyRune(() => Settings.getAsync());
Settings.onChange((s) => (_settings.value = s));
await _settings.init();

$effect.root(() => {
  $effect(() => {
    locale.set(_settings.value.language || "en");
  });
});

export const settingsState = {
  get value() {
    return _settings.value.byWidget["@seelen/fancy-toolbar"] as any;
  },
  get language() {
    return (_settings.value.language || "en") as string;
  },
  get dateFormat() {
    return _settings.value.dateFormat as string;
  },
  get startOfWeek() {
    return _settings.value.startOfWeek;
  },
  get allByWidget() {
    return _settings.value.byWidget;
  },
  get itemSize(): number {
    return (this.value?.itemSize ?? 24) as number;
  },
  get margin(): number {
    return (this.value?.margin ?? 0) as number;
  },
  get padding(): number {
    return (this.value?.padding ?? 4) as number;
  },
  get position(): FancyToolbarSide {
    return (this.value?.position ?? FancyToolbarSide.Top) as FancyToolbarSide;
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
  const rect = widgetRect.value;
  const hideMode = settingsState.hideMode;
  const position = settingsState.position;

  await Widget.self.setPosition(rect);

  if (hideMode === HideMode.Never || isTouchPrimary.value) {
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
    // track reactive deps
    settingsState.position;
    settingsState.hideMode;
    settingsState.itemSize;
    settingsState.margin;
    settingsState.padding;
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
