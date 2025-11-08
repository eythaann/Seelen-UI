// before tauri v2.5 this script was done as a workaround to https://github.com/tauri-apps/tauri/issues/12348
// but was fixed on https://github.com/tauri-apps/wry/pull/1531 so now this script is used to initialize
// the console logger to capture any error on the main script

import { wrapConsoleV2 } from "./ConsoleWrapper.ts";
wrapConsoleV2();

import type { Widget } from "@seelen-ui/lib/types";
import { _invoke, WebviewInformation } from "libs/widgets-integrity/_tauri";

import { removeDefaultWebviewActions } from "@shared/setup.ts";

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
