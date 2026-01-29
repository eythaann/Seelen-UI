import { getRootContainer } from "libs/ui/react/utils/index.ts";
import { disableAnimationsOnPerformanceMode } from "libs/ui/react/utils/performance.ts";
import { createRoot } from "react-dom/client";
import { I18nextProvider } from "react-i18next";
import { Provider } from "react-redux";

import { registerStoreEvents, store } from "./modules/shared/store/infra.ts";

import { App } from "./app.tsx";
import { Widget } from "@seelen-ui/lib";

import i18n, { loadTranslations } from "./i18n/index.ts";

import "@shared/styles/colors.css";
import "./styles/variables.css";
import "@shared/styles/reset.css";
import "./styles/global.css";

await registerStoreEvents();
await loadTranslations();
disableAnimationsOnPerformanceMode();

await Widget.getCurrent().init();

const container = getRootContainer();
createRoot(container).render(
  <Provider store={store}>
    <I18nextProvider i18n={i18n}>
      <App />
    </I18nextProvider>
  </Provider>,
);
