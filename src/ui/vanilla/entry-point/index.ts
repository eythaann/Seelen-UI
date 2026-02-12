// before tauri v2.5 this script was done as a workaround to https://github.com/tauri-apps/tauri/issues/12348
// but was fixed on https://github.com/tauri-apps/wry/pull/1531 so now this script is used to initialize
// the console logger to capture any error on the main script

import "./ConsoleWrapper.ts";

import type { FocusedApp, Widget } from "@seelen-ui/lib/types";
import { emitTo, listen } from "@tauri-apps/api/event";
import { _invoke, WebviewInformation } from "src/ui/vanilla/entry-point/_tauri.ts";

import { hookLocalStorage, removeDefaultWebviewActions } from "src/ui/vanilla/entry-point/setup.ts";

const indexJsCode = fetch("./index.js").then((res) => res.text());

// initialize global widget variable, needed by slu-lib
const info = new WebviewInformation();
const currentWidgetId = info.widgetId;
const widgetList = await _invoke<Widget[]>("state_get_widgets");
window.__SLU_WIDGET = widgetList.find((widget) => widget.id === currentWidgetId)!;

if (!window.__SLU_WIDGET) {
  throw new Error(`Widget definition not found for ${currentWidgetId}`);
}

// remove default browser actions, we don't need them
removeDefaultWebviewActions();
hookLocalStorage(currentWidgetId);

// trigger garbage collection
setTimeout(() => {
  window.gc?.();
}, 1000);

if (!window.__SLU_WIDGET.noMemoryLeakWorkaround) {
  // workaround for tauri/webview2 memory leak
  setInterval(async () => {
    const app = await _invoke<FocusedApp>("get_focused_app");
    // avoid reload the UI while playing as this can cause fps drops
    if (app.isFullscreened || app.exe?.endsWith("seelen-ui.exe")) {
      return;
    }

    console.trace("Reloading widget.");
    location.search = `r=${Date.now()}`; // add a query hash to force be a new page
  }, 60_000 * 10); // every 10 minutes
}

listen("internal::session_resumed", () => {
  console.trace("Reloading widget.");
  location.search = `r=${Date.now()}`; // add a query hash to force be a new page
});

listen<string>(
  "internal::liveness-ping",
  () => {
    emitTo(info.rawLabel, "internal::liveness-pong");
  },
  {
    target: {
      kind: "WebviewWindow",
      label: info.rawLabel,
    },
  },
);

// load index.js
const script = document.createElement("script");
script.type = "module";
script.textContent = await indexJsCode;
document.head.appendChild(script);
