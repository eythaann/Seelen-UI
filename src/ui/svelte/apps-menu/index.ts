import { getRootContainer } from "libs/ui/react/utils/index.ts";
import { mount } from "svelte";
import App from "./App.svelte";
import { loadTranslations } from "./i18n/index.ts";
import { Widget } from "@seelen-ui/lib";
import { onTriggered } from "./state/positioning.svelte.ts";

import "@seelen-ui/lib/styles/reset.css";

await loadTranslations();

const widget = Widget.getCurrent();

await widget.init({ hideOnFocusLoss: true });

let lastFocusLossAt = 0;
widget.window.onFocusChanged(({ payload: focused }) => {
  if (!focused) {
    lastFocusLossAt = Date.now();
  }
});

widget.onTrigger(async (args) => {
  const visible = await widget.window.isVisible();
  const recentlyHiddenByFocusLoss = Date.now() - lastFocusLossAt < 300;
  if (visible || recentlyHiddenByFocusLoss) {
    widget.hide();
  } else {
    onTriggered(args.monitorId);
  }
});

mount(App, {
  target: getRootContainer(),
});
