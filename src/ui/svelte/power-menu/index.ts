import { getRootContainer } from "libs/ui/react/utils/index.ts";
import { mount } from "svelte";
import App from "./app.svelte";
import { loadTranslations } from "./i18n/index.ts";
import { Widget } from "@seelen-ui/lib";

import "@seelen-ui/lib/styles/reset.css";

await loadTranslations();

const widget = Widget.getCurrent();
let lastFocusLossAt = 0;
widget.window.onFocusChanged(({ payload: focused }) => {
  if (!focused) {
    lastFocusLossAt = Date.now();
  }
});

widget.onTrigger(async () => {
  const recentlyHiddenByFocusLoss = Date.now() - lastFocusLossAt < 300;
  if ((await widget.window.isVisible()) || recentlyHiddenByFocusLoss) {
    widget.hide();
    return;
  }

  await widget.show();
  await widget.focus();
});

await widget.init({ normalizeDevicePixelRatio: true, hideOnFocusLoss: true });
await widget.window.setResizable(false);

mount(App, {
  target: getRootContainer(),
});
