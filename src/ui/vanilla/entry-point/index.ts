// before tauri v2.5 this script was done as a workaround to https://github.com/tauri-apps/tauri/issues/12348
// but was fixed on https://github.com/tauri-apps/wry/pull/1531 so now this script is used to initialize
// the console logger to capture any error on the main script

import "./ConsoleWrapper.ts";

import type { FocusedApp, Widget } from "@seelen-ui/lib/types";
import { listen } from "@tauri-apps/api/event";
import { _invoke, WebviewInformation } from "src/ui/vanilla/entry-point/_tauri.ts";

import { removeDefaultWebviewActions } from "src/ui/vanilla/entry-point/setup.ts";

const indexJsCode = fetch("./index.js").then((res) => res.text());

// initialize global widget variable, needed by slu-lib
const currentWidgetId = new WebviewInformation().widgetId;
const widgetList = await _invoke<Widget[]>("state_get_widgets");
window.__SLU_WIDGET = widgetList.find((widget) => widget.id === currentWidgetId)!;

if (!window.__SLU_WIDGET) {
  throw new Error(`Widget definition not found for ${currentWidgetId}`);
}

// load index.js
const script = document.createElement("script");
script.type = "module";
script.textContent = await indexJsCode;
document.head.appendChild(script);

// remove default browser actions, we don't need them
removeDefaultWebviewActions();

// trigger garbage collection
setTimeout(() => {
  window.gc?.();
}, 1000);

if (!window.__SLU_WIDGET.noMemoryLeakWorkaround) {
  // workaround for tauri/webview2 memory leak
  setTimeout(async () => {
    const app = await _invoke<FocusedApp>("get_focused_app");
    // avoid reload the UI while playing as this can cause fps drops
    if (app.isFullscreened) {
      return;
    }

    if (!app.exe?.endsWith("seelen-ui.exe")) {
      console.trace("Reloading widget.");
      location.reload();
    }
  }, 60_000 * 10); // every 10 minutes
}

listen("internal::session_resumed", () => {
  console.trace("Reloading widget.");
  location.reload();
});
