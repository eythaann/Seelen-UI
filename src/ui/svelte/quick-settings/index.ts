import { mount } from "svelte";
import App from "./app.svelte";
import { loadTranslations } from "./i18n";
import { Widget } from "@seelen-ui/lib";

import "@seelen-ui/lib/styles/reset.css";

await loadTranslations();

const root = document.getElementById("root")!;

const widget = Widget.getCurrent();
await widget.init({
  autoSizeByContent: root,
});

mount(App, {
  target: root,
});
