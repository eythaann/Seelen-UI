import { getRootContainer } from "@shared/index";
import { declareDocumentAsLayeredHitbox } from "@shared/layered";
import { disableAnimationsOnPerformanceMode } from "@shared/performance";
import { removeDefaultWebviewActions } from "@shared/setup";
import { createRoot } from "react-dom/client";
import { I18nextProvider } from "react-i18next";
import { Provider } from "react-redux";

import { initStore, store } from "./modules/shared/store/infra.ts";

import { App } from "./App.tsx";
import { registerDocumentEvents } from "./events.ts";
import i18n, { loadTranslations } from "./i18n/index.ts";

import "@shared/styles/reset.css";
import "@shared/styles/colors.css";

removeDefaultWebviewActions();
await declareDocumentAsLayeredHitbox();
await loadTranslations();
await initStore();
registerDocumentEvents();
disableAnimationsOnPerformanceMode();

createRoot(getRootContainer()).render(
  <Provider store={store}>
    <I18nextProvider i18n={i18n}>
      <App />
    </I18nextProvider>
  </Provider>,
);
