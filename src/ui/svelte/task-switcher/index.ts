import { disableAnimationsOnPerformanceMode } from "libs/ui/react/utils/performance";
import { mount } from "svelte";
import App from "./App.svelte";
import { Widget } from "@seelen-ui/lib";

import "@shared/styles/colors.css";
import "@shared/styles/reset.css";

const root = document.getElementById("root")!;

const widget = Widget.getCurrent();
await widget.init({
  show: false,
});
await widget.webview.setResizable(false);

disableAnimationsOnPerformanceMode();

mount(App, {
  target: root,
});
