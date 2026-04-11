import { mount } from "svelte";
import App from "./App.svelte";
import { Widget } from "@seelen-ui/lib";

import "@seelen-ui/lib/styles/reset.css";

const root = document.getElementById("root")!;

const widget = Widget.getCurrent();
await widget.init();
await widget.window.setResizable(false);

mount(App, {
  target: root,
});
