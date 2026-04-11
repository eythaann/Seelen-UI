// before tauri v2.5 this script was done as a workaround to https://github.com/tauri-apps/tauri/issues/12348
// but was fixed on https://github.com/tauri-apps/wry/pull/1531 so now this script is used to initialize the widgets system

import "@seelen-ui/lib/styles/colors.css";

import "./ConsoleWrapper.ts";
import "./MainSetup.ts";
import "./MemoryLeakWorkaround.ts";
import "./PreventDefaults.ts";
import "./UxImprovements.ts";
