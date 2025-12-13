import { mount } from "svelte";
import App from "./app.svelte";
import { loadTranslations } from "./i18n/index.ts";
import { Widget } from "@seelen-ui/lib";

import "@shared/styles/reset.css";
import "@shared/styles/colors.css";

const root = document.getElementById("root")!;

const widget = Widget.getCurrent();
await widget.init({
  autoSizeByContent: root,
});

await loadTranslations();

mount(App, {
  target: root,
});
