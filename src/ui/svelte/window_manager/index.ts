import { mount } from "svelte";
import { Widget } from "@seelen-ui/lib";
import { getRootContainer } from "libs/ui/react/utils";
import { declareDocumentAsLayeredHitbox } from "libs/ui/react/utils/layered";
import { disableAnimationsOnPerformanceMode } from "libs/ui/react/utils/performance";

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
