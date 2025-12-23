import { Widget } from "@seelen-ui/lib";
import { getRootContainer } from "libs/ui/react/utils/index.ts";
import { disableAnimationsOnPerformanceMode } from "libs/ui/react/utils/performance.ts";
import { createRoot } from "react-dom/client";
import { I18nextProvider } from "react-i18next";

import { App } from "./app.tsx";

import i18n, { loadTranslations } from "./i18n/index.ts";

import "@shared/styles/colors.css";
import "@shared/styles/reset.css";
import "./styles/global.css";

await Widget.getCurrent().init();

await loadTranslations();
disableAnimationsOnPerformanceMode();

const container = getRootContainer();
createRoot(container).render(
  <I18nextProvider i18n={i18n}>
    <App />
  </I18nextProvider>,
);
