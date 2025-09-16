import { startThemingTool } from "@seelen-ui/lib";
import { getRootContainer } from "@shared/index";
import { disableAnimationsOnPerformanceMode } from "@shared/performance";
import { createRoot } from "react-dom/client";

import { App } from "./app";

import "@shared/styles/reset.css";
import "@shared/styles/colors.css";
import "./global.css";

await startThemingTool();
disableAnimationsOnPerformanceMode();

const container = getRootContainer();
createRoot(container).render(<App />);
