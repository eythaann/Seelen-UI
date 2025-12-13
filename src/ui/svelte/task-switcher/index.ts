import { getRootContainer } from "@shared";
import { declareDocumentAsLayeredHitbox } from "@shared/layered";
import { disableAnimationsOnPerformanceMode } from "@shared/performance";
import { mount } from "svelte";
import App from "./App.svelte";
import { Widget } from "@seelen-ui/lib";

import "@shared/styles/colors.css";
import "@shared/styles/reset.css";

disableAnimationsOnPerformanceMode();
await declareDocumentAsLayeredHitbox();

const widget = Widget.getCurrent();
await widget.init();
widget.autoSizeWebviewByElement(getRootContainer());

mount(App, {
  target: getRootContainer(),
});
