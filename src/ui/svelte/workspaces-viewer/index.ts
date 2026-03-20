import { mount } from "svelte";
import App from "./app.svelte";
import { Widget } from "@seelen-ui/lib";

import "@shared/styles/reset.css";
import "@shared/styles/colors.css";

const widget = Widget.getCurrent();
await widget.init();

// play with zoom level to reset device pixel ratio to 1:1
let lastDPR = window.devicePixelRatio;
await widget.webview.setZoom(1 / lastDPR);
widget.window.onScaleChanged(() => {
  if (window.devicePixelRatio !== lastDPR) {
    // when zoom was set dpr changed, so in case of change this is accomulative unit
    lastDPR = lastDPR * window.devicePixelRatio;
    widget.webview.setZoom(1 / (lastDPR * window.devicePixelRatio));
  }
});

widget.window.setResizable(false);
widget.onTrigger(async () => {
  if (await widget.window.isVisible()) {
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
