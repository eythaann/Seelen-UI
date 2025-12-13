import { getRootContainer } from "@shared";
import { declareDocumentAsLayeredHitbox } from "@shared/layered";
import { disableAnimationsOnPerformanceMode } from "@shared/performance";
import { render } from "preact";
import { App } from "./app.tsx";
import { Widget } from "@seelen-ui/lib";

import "@shared/styles/colors.css";
import "@shared/styles/reset.css";
import "./index.css";

disableAnimationsOnPerformanceMode();
await declareDocumentAsLayeredHitbox();

const widget = Widget.getCurrent();
await widget.init();

const container = getRootContainer();
render(<App />, container);
