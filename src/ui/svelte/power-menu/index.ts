import { getRootContainer } from "libs/ui/react/utils/index.ts";
import { mount } from "svelte";
import App from "./app.svelte";
import { loadTranslations } from "./i18n/index.ts";
import { Widget } from "@seelen-ui/lib";

import "@seelen-ui/lib/styles/reset.css";

await loadTranslations();

const widget = Widget.getCurrent();
widget.onTrigger(async () => {
  await widget.show();
  await widget.focus();
});

await widget.init({ normalizeDevicePixelRatio: true, hideOnFocusLoss: true });
await widget.window.setResizable(false);

mount(App, {
  target: getRootContainer(),
});
