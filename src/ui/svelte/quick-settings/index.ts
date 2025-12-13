import { mount } from "svelte";
import App from "./app.svelte";
import { Widget } from "@seelen-ui/lib";

import "@shared/styles/reset.css";
import "@shared/styles/colors.css";

const widget = Widget.getCurrent();
await widget.init();

let root = document.getElementById("root")!;
widget.autoSizeWebviewByElement(root);

mount(App, {
  target: root,
});
