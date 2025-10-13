import { startThemingTool } from "@seelen-ui/lib";
import { getRootContainer } from "@shared";
import { disableAnimationsOnPerformanceMode } from "@shared/performance";
import { removeDefaultWebviewActions } from "@shared/setup";
import { createRoot } from "react-dom/client";

import { App } from "./app.tsx";

import "@shared/styles/colors.css";
import "@shared/styles/reset.css";
import "./styles/global.css";

removeDefaultWebviewActions();
await startThemingTool();
disableAnimationsOnPerformanceMode();

const container = getRootContainer();
createRoot(container).render(<App />);
