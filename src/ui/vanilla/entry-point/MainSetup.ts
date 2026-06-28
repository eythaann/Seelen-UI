import type { Widget } from "@seelen-ui/lib/types";
import { _invoke, webviewInfo } from "src/ui/vanilla/entry-point/_tauri.ts";
import { listen } from "@tauri-apps/api/event";

import { hookLocalStorage } from "src/ui/vanilla/entry-point/_LocalStorage";

const indexJsCode = fetch("./index.js").then((res) => res.text());

// initialize global widget variable, needed by slu-lib
const currentWidgetId = webviewInfo.widgetId;
const widgetList = await _invoke<Widget[]>("state_get_widgets");
window.__SLU_WIDGET = widgetList.find((widget) => widget.id === currentWidgetId)!;

if (!window.__SLU_WIDGET) {
  throw new Error(`Widget definition not found for ${currentWidgetId}`);
}

// reload if widget definition changed
listen<Widget[]>("widgets-changed", ({ payload }) => {
  const actual = payload.find((widget) => widget.id === currentWidgetId);
  if (actual && JSON.stringify(actual) !== JSON.stringify(window.__SLU_WIDGET)) {
    window.location.reload();
  }
});

// set document id
document.documentElement.id = currentWidgetId;

// hook local storage, to avoid collition of keys
hookLocalStorage(currentWidgetId);

// add base css
const link = document.createElement("link");
link.rel = "stylesheet";
link.href = "/vanilla/entry-point/index.css";
document.head.appendChild(link);

// load index.js
const script = document.createElement("script");
script.type = "module";
script.textContent = await indexJsCode;
document.head.appendChild(script);
