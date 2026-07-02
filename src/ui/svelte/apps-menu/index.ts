import { getRootContainer } from "libs/ui/react/utils/index.ts";
import { mount } from "svelte";
import App from "./App.svelte";
import { Widget } from "@seelen-ui/lib";
import { onTriggered } from "./state/positioning.svelte.ts";

import "@seelen-ui/lib/styles/reset.css";

const widget = Widget.getCurrent();

await widget.init({ hideOnFocusLoss: true });
await widget.window.setFocusable(true);

widget.onTrigger(async (args) => {
  const visible = await widget.window.isVisible();
  if (visible) {
    widget.hide();
  } else {
    onTriggered(args.monitorId, args.desiredPosition);
  }
});

mount(App, {
  target: getRootContainer(),
});
