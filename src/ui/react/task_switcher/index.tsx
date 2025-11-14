import { startThemingTool } from "@seelen-ui/lib";
import { getRootContainer } from "@shared";
import { declareDocumentAsLayeredHitbox } from "@shared/layered";
import { disableAnimationsOnPerformanceMode } from "@shared/performance";
import { render } from "preact";
import { App } from "./app.tsx";

import "@shared/styles/colors.css";
import "@shared/styles/reset.css";
import "./index.css";

disableAnimationsOnPerformanceMode();
await declareDocumentAsLayeredHitbox();
await startThemingTool();

const container = getRootContainer();
render(<App />, container);
