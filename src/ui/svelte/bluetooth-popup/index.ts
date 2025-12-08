import { mount } from "svelte";
import App from "./app.svelte";
import { startThemingTool } from "libs/core/npm/esm/mod";
import { loadTranslations } from "./i18n/index.ts";
import { Widget } from "@seelen-ui/lib";

import "@shared/styles/reset.css";
import "@shared/styles/colors.css";

await loadTranslations();
await startThemingTool();

const widget = Widget.getCurrent();
await widget.init();

let root = document.getElementById("root")!;
widget.autoSizeWebviewByElement(root);

mount(App, {
  target: root,
});
