import { getRootContainer } from "libs/ui/react/utils/index.ts";
import { mount } from "svelte";
import App from "./app.svelte";
import { loadTranslations } from "./i18n/index.ts";
import { Widget } from "@seelen-ui/lib";

import "@shared/styles/reset.css";
import "@shared/styles/colors.css";

await loadTranslations();

const widget = Widget.getCurrent();
await widget.init();

// play with zoom level to reset device pixel ratio to 1:1
await widget.webview.setZoom(1 / (await widget.webview.scaleFactor()));
widget.webview.onScaleChanged(({ payload }) => {
  widget.webview.setZoom(1 / payload.scaleFactor);
});

widget.webview.setResizable(false);
widget.webview.onFocusChanged((e) => {
  if (!e.payload) {
    widget.hide(true);
  }
});
widget.onTrigger(async () => {
  await widget.show();
  await widget.focus();
});

mount(App, {
  target: getRootContainer(),
});
