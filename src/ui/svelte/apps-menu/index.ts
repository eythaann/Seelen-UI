import { getRootContainer } from "libs/ui/react/utils/index.ts";
import { mount } from "svelte";
import App from "./App.svelte";
import { loadTranslations } from "./i18n/index.ts";
import { Widget } from "@seelen-ui/lib";
import { onTriggered } from "./positioning.svelte.ts";

import "@shared/styles/reset.css";
import "@shared/styles/colors.css";
import { Effect } from "@tauri-apps/api/window";
import { debounce } from "lodash";
import { globalState } from "./state.svelte.ts";

await loadTranslations();

const widget = Widget.getCurrent();
const { webview } = widget;

await widget.init();

await Promise.all([
  webview.setDecorations(false),
  webview.setMinimizable(false),
  webview.setClosable(false),
  webview.setSkipTaskbar(true),
  webview.setEffects({
    effects: [Effect.Acrylic],
  }),
  webview.setAlwaysOnTop(true),
  webview.setResizable(false),
]);

const hideDelayed = debounce(() => {
  globalState.showing = false;
}, 100);

widget.webview.onFocusChanged((e) => {
  if (e.payload) {
    hideDelayed.cancel();
  } else {
    hideDelayed();
  }
});

widget.onTrigger(async (args) => {
  const visible = await widget.webview.isVisible();
  if (visible) {
    globalState.showing = false;
  } else {
    onTriggered(args.monitorId);
  }
});

mount(App, {
  target: getRootContainer(),
});
