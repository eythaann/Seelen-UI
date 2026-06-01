import { mount } from "svelte";
import App from "./app.svelte";
import { loadTranslations } from "./i18n";
import { Widget } from "@seelen-ui/lib";
import { LogicalSize } from "@seelen-ui/lib/tauri";

import "@seelen-ui/lib/styles/reset.css";

await loadTranslations();
await Widget.self.init();
await Promise.all([
  Widget.self.window.setSizeConstraints({ minWidth: 600, minHeight: 400 }),
  Widget.self.window.setSize(new LogicalSize(800, 500)),
  Widget.self.window.center(),
]);

mount(App, {
  target: document.getElementById("root")!,
});
