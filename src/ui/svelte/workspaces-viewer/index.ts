import { mount } from "svelte";
import App from "./app.svelte";
import { startThemingTool, Widget } from "@seelen-ui/lib";
// import { loadTranslations } from "./i18n/index.ts";

import "@shared/styles/reset.css";
import "@shared/styles/colors.css";

// await loadTranslations();
await startThemingTool();

const widget = Widget.getCurrent();
await widget.init();

widget.webview.setResizable(false);
widget.onTrigger(() => {
  widget.webview.show();
});

const root = document.getElementById("root")!;
mount(App, {
  target: root,
});
