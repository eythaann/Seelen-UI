import { mount } from "svelte";
import App from "./app.svelte";
import { Widget } from "@seelen-ui/lib";

import "@shared/styles/reset.css";
import "@shared/styles/colors.css";

const widget = Widget.getCurrent();
await widget.init();

// play with zoom level to reset device pixel ratio to 1:1
await widget.webview.setZoom(1 / (await widget.webview.scaleFactor()));
widget.webview.onScaleChanged(({ payload }) => {
  widget.webview.setZoom(1 / payload.scaleFactor);
});

widget.webview.setResizable(false);
widget.onTrigger(async () => {
  if (await widget.webview.isVisible()) {
    widget.hide(true);
  } else {
    await widget.show();
    await widget.focus();
  }
});

const root = document.getElementById("root")!;
mount(App, {
  target: root,
});
