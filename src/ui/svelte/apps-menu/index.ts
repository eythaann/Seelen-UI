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

widget.onTrigger(async (args) => {
  const visible = await widget.window.isVisible();
  if (visible) {
    widget.hide();
  } else {
    onTriggered(args.monitorId);
  }
});

mount(App, {
  target: getRootContainer(),
});
