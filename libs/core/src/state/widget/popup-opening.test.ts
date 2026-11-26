/// <reference lib="deno.ns" />

import { assert, assertEquals } from "@std/assert";
import { Alignment, type Widget as WidgetDefinition, WidgetPreset } from "@seelen-ui/types";

const callbacks: Array<(event: { payload: unknown }) => void> = [];
let triggerPopup: typeof callbacks[number] | undefined;
const nativeCalls: string[] = [];
let currentRect = { left: 0, top: 0, right: 400, bottom: 300 };
let hiddenAtShow: boolean | undefined;
const dataset: Record<string, string> = {};
const content = { scrollWidth: 400, scrollHeight: 300 } as HTMLElement;

const testWindow = {
  metadata: {
    currentWindow: { label: "popup-test" },
    currentWebview: { label: "QHNlZWxlbi93ZWctcHJldmlldw" },
  },
  transformCallback(callback: (event: { payload: unknown }) => void): number {
    return callbacks.push(callback) - 1;
  },
  invoke(command: string, args?: Record<string, unknown>): unknown {
    if (command === "plugin:event|listen") {
      if (args?.event === "widget::triggered") triggerPopup = callbacks[Number(args.handler)];
      return Promise.resolve(0);
    }
    if (command === "plugin:window|is_visible") return Promise.resolve(false);
    if (command === "plugin:window|outer_size") {
      return Promise.resolve({
        width: currentRect.right - currentRect.left,
        height: currentRect.bottom - currentRect.top,
      });
    }
    if (command === "plugin:window|outer_position") {
      return Promise.resolve({ x: currentRect.left, y: currentRect.top });
    }
    if (command === "plugin:window|set_focusable") {
      nativeCalls.push("focusable");
      return Promise.resolve();
    }
    if (command === "plugin:window|show") {
      hiddenAtShow = "widgetHidden" in dataset;
      nativeCalls.push("show");
      return Promise.resolve();
    }
    if (command === "set_self_position") {
      currentRect = args?.rect as typeof currentRect;
      nativeCalls.push("resize");
      return Promise.resolve();
    }
    if (command === "get_self_window_handle") return Promise.resolve("17");
    if (command === "state_get_performance_mode") return Promise.resolve("Disabled");
    if (command === "get_connected_monitors") {
      return Promise.resolve([{ rect: { left: 0, top: 0, right: 1920, bottom: 1080 }, isPrimary: true }]);
    }
    if (["request_focus", "set_current_widget_status", "plugin:window|set_resizable"].includes(command)) {
      return Promise.resolve();
    }
    throw new Error(`Unexpected command: ${command}`);
  },
};

Object.assign(globalThis, {
  window: { __TAURI_INTERNALS__: testWindow, devicePixelRatio: 1 },
  document: {
    body: {},
    visibilityState: "visible",
    documentElement: {
      dataset,
      toggleAttribute(name: string, present: boolean): void {
        if (name !== "data-widget-hidden") return;
        if (present) dataset.widgetHidden = "";
        else delete dataset.widgetHidden;
      },
    },
  },
  getComputedStyle: () => ({ opacity: "0" }),
  requestAnimationFrame: (callback: () => void) => {
    callback();
    return 0;
  },
  cancelAnimationFrame: () => {},
});

const { Widget } = await import("./mod.ts");
const channel = {
  subscribe(): Promise<() => void> {
    return Promise.resolve(() => {});
  },
};
(globalThis as unknown as Record<symbol, unknown>)[Symbol.for("@seelen-ui/lib/events-channel")] = channel;

Deno.test("changed popup content is sized and fading before the native window is shown", async () => {
  globalThis.ResizeObserver = class {
    observe(): void {}
    unobserve(): void {}
    disconnect(): void {}
  } as typeof ResizeObserver;
  globalThis.__SLU_WIDGET = {
    id: "@seelen/weg-preview",
    preset: WidgetPreset.Popup,
    lazy: true,
  } as WidgetDefinition;
  const widget = Widget.self;
  await widget.init({
    autoSizeByContent: content,
    useThemes: false,
    disableCssAnimations: true,
    hideOnFocusLoss: false,
  });
  await widget.ready();

  Object.defineProperty(content, "scrollWidth", { value: 600 });
  Object.defineProperty(content, "scrollHeight", { value: 400 });
  nativeCalls.length = 0;

  assert("widgetHidden" in dataset);
  assert(triggerPopup);
  triggerPopup({
    payload: {
      desiredPosition: { x: 800, y: 900 },
      alignX: Alignment.Center,
      alignY: Alignment.End,
      source: "Click",
    },
  });
  await new Promise((resolve) => setTimeout(resolve, 0));

  assertEquals(nativeCalls, ["resize", "resize", "show"]);
  assertEquals(hiddenAtShow, false);
  assertEquals(currentRect, { left: 500, top: 500, right: 1100, bottom: 900 });
});

declare global {
  var __SLU_WIDGET: WidgetDefinition;
  var __SLU_WIDGET_INSTANCE: ReturnType<typeof Widget.getCurrent> | undefined;
}
