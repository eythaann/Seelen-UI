import { declareDocumentAsLayeredHitbox } from "@shared/layered";
import { disableAnimationsOnPerformanceMode } from "@shared/performance";
import { mount } from "svelte";
import App from "./App.svelte";
import { Widget } from "@seelen-ui/lib";

import "@shared/styles/colors.css";
import "@shared/styles/reset.css";

const root = document.getElementById("root")!;

const widget = Widget.getCurrent();
await widget.init({
  show: false,
  autoSizeByContent: root,
});

disableAnimationsOnPerformanceMode();
await declareDocumentAsLayeredHitbox();

mount(App, {
  target: root,
});
