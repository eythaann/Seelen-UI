import { mount } from "svelte";
import App from "./app.svelte";
import { Widget } from "@seelen-ui/lib";

import "@seelen-ui/lib/styles/reset.css";

const widget = Widget.getCurrent();
await widget.init({ normalizeDevicePixelRatio: true });

widget.window.setResizable(false);
widget.onTrigger(async () => {
  if (await widget.window.isVisible()) {
    widget.hide();
  } else {
    await widget.show();
    await widget.focus();
  }
});

const root = document.getElementById("root")!;
mount(App, {
  target: root,
});
