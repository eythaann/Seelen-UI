import { mount } from "svelte";
import App from "./app.svelte";
import { Widget } from "@seelen-ui/lib";

import "@shared/styles/reset.css";
import "@shared/styles/colors.css";

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
