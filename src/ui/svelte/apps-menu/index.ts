import { getRootContainer } from "libs/ui/react/utils/index.ts";
import { mount } from "svelte";
import App from "./App.svelte";
import { loadTranslations } from "./i18n/index.ts";
import { Widget } from "@seelen-ui/lib";
import { onTriggered } from "./state/positioning.svelte.ts";

import "@shared/styles/reset.css";
import "@shared/styles/colors.css";
import { debounce } from "lodash";
import { globalState } from "./state/mod.svelte.ts";

await loadTranslations();

const widget = Widget.getCurrent();
const { window } = widget;

await widget.init();

await Promise.all([
  window.setDecorations(false),
  window.setMinimizable(false),
  window.setMaximizable(false),
  window.setClosable(false),
  window.setSkipTaskbar(true),
  window.setAlwaysOnTop(true),
  window.setResizable(false),
]);

const hideDelayed = debounce(() => {
  globalState.showing = false;
}, 100);

widget.window.onFocusChanged((e) => {
  if (e.payload) {
    hideDelayed.cancel();
  } else {
    hideDelayed();
  }
});

widget.onTrigger(async (args) => {
  const visible = await widget.window.isVisible();
  if (visible) {
    globalState.showing = false;
  } else {
    onTriggered(args.monitorId);
  }
});

mount(App, {
  target: getRootContainer(),
});
