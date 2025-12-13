import { mount } from "svelte";
import { Widget } from "@seelen-ui/lib";
import { getRootContainer } from "@shared";
import { declareDocumentAsLayeredHitbox } from "@shared/layered";
import { disableAnimationsOnPerformanceMode } from "@shared/performance";

import App from "./App.svelte";

import "@shared/styles/colors.css";
import "@shared/styles/reset.css";
import "./styles/global.css";

const widget = Widget.getCurrent();
await widget.init();

await declareDocumentAsLayeredHitbox((e) => e.getAttribute("data-allow-mouse-events") === "true");
disableAnimationsOnPerformanceMode();

const container = getRootContainer();
mount(App, {
  target: container,
});
