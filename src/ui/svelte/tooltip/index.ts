import { mount } from "svelte";
import App from "./app.svelte";
import { Widget } from "@seelen-ui/lib";

import "@seelen-ui/lib/styles/reset.css";

const root = document.getElementById("root")!;

await Widget.self.init({
  autoSizeByContent: root,
});

await Widget.self.window.setIgnoreCursorEvents(true);

mount(App, {
  target: root,
});
