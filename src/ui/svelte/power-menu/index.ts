import { getRootContainer } from "libs/ui/react/utils/index.ts";
import { mount } from "svelte";
import App from "./app.svelte";
import { loadTranslations } from "./i18n/index.ts";
import { Widget } from "@seelen-ui/lib";

import "@shared/styles/reset.css";
import "@shared/styles/colors.css";
import { debounce } from "lodash";

await loadTranslations();

const widget = Widget.getCurrent();
widget.onTrigger(async () => {
  await widget.show();
  await widget.focus();
});

const hide = debounce(() => {
  widget.hide(true);
}, 100);

widget.webview.onFocusChanged(({ payload: focused }) => {
  if (focused) {
    hide.cancel();
  } else {
    hide();
  }
});

await widget.init();
await widget.webview.setResizable(false);

// play with zoom level to reset device pixel ratio to 1:1
await widget.webview.setZoom(1 / (await widget.webview.scaleFactor()));
widget.webview.onScaleChanged(({ payload }) => {
  widget.webview.setZoom(1 / payload.scaleFactor);
});

mount(App, {
  target: getRootContainer(),
});
