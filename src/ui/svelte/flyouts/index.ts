import { mount } from "svelte";
import App from "./app.svelte";
import { loadTranslations } from "./i18n/index.ts";
import { Widget } from "@seelen-ui/lib";

import "@seelen-ui/lib/styles/reset.css";

await loadTranslations();

const root = document.getElementById("root")!;

await Widget.self.init({
  autoSizeByContent: root,
});

mount(App, {
  target: root,
});
