import { Widget } from "@seelen-ui/lib";
import { getRootContainer } from "@shared/index";
import { createRoot } from "react-dom/client";

import { App } from "./app.tsx";

import "@shared/styles/reset.css";
import "@shared/styles/colors.css";
import "./global.css";

await Widget.getCurrent().init();

const container = getRootContainer();
createRoot(container).render(<App />);
