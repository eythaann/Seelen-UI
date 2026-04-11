import { mount } from "svelte";
import App from "./app.svelte";
import { Widget } from "@seelen-ui/lib";

import "@seelen-ui/lib/styles/reset.css";

const root = document.getElementById("root")!;

const widget = Widget.getCurrent();
await widget.init({
  autoSizeByContent: root,
});

mount(App, {
  target: root,
});
