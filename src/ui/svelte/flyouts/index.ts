import { mount } from "svelte";
import App from "./app.svelte";
import { Widget } from "@seelen-ui/lib";

import "@shared/styles/reset.css";
import "@shared/styles/colors.css";

const root = document.getElementById("root")!;

await Widget.self.init({
  autoSizeByContent: root,
  show: false,
});
await Widget.self.webview.setFocusable(false);

mount(App, {
  target: root,
});
