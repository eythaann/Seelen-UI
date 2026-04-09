import type { Widget } from "@seelen-ui/lib/types";
import { _invoke, webviewInfo } from "src/ui/vanilla/entry-point/_tauri.ts";

import { hookLocalStorage } from "src/ui/vanilla/entry-point/_LocalStorage";

const indexJsCode = fetch("./index.js").then((res) => res.text());

// initialize global widget variable, needed by slu-lib
const currentWidgetId = webviewInfo.widgetId;
const widgetList = await _invoke<Widget[]>("state_get_widgets");
window.__SLU_WIDGET = widgetList.find((widget) => widget.id === currentWidgetId)!;

if (!window.__SLU_WIDGET) {
  throw new Error(`Widget definition not found for ${currentWidgetId}`);
}

// hook local storage, to avoid collition of keys
hookLocalStorage(currentWidgetId);

// load index.js
const script = document.createElement("script");
script.type = "module";
script.textContent = await indexJsCode;
document.head.appendChild(script);
