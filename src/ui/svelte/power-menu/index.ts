import { getRootContainer } from "@shared";
import { mount } from "svelte";
import App from "./app.svelte";
import { loadTranslations } from "./i18n/index.ts";
import { Widget } from "@seelen-ui/lib";

import "@shared/styles/reset.css";
import "@shared/styles/colors.css";

await loadTranslations();

const widget = Widget.getCurrent();
await widget.init();

mount(App, {
  target: getRootContainer(),
});
