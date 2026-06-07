import { Widget } from "@seelen-ui/lib";
import { getRootContainer } from "libs/ui/react/utils/index";
import { createRoot } from "react-dom/client";

import { App } from "./app.tsx";

import "@seelen-ui/lib/styles/reset.css";
import "./global.css";

await Widget.getCurrent().init();

const container = getRootContainer();
createRoot(container).render(<App />);
