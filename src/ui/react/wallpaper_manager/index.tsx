import { Widget } from "@seelen-ui/lib";
import { getRootContainer } from "@shared";
import { disableAnimationsOnPerformanceMode } from "@shared/performance";
import { createRoot } from "react-dom/client";

import { App } from "./app.tsx";

import "@shared/styles/colors.css";
import "@shared/styles/reset.css";
import "./styles/global.css";

disableAnimationsOnPerformanceMode();
await Widget.getCurrent().init();

const container = getRootContainer();
createRoot(container).render(<App />);
