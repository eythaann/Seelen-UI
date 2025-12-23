import { getRootContainer } from "libs/ui/react/utils/index";
import { declareDocumentAsLayeredHitbox } from "libs/ui/react/utils/layered.ts";
import { disableAnimationsOnPerformanceMode } from "libs/ui/react/utils/performance.ts";
import { createRoot } from "react-dom/client";
import { I18nextProvider } from "react-i18next";
import { Provider } from "react-redux";

import { initStore, store } from "./modules/shared/store/infra.ts";

import { App } from "./App.tsx";
import { registerDocumentEvents } from "./events.ts";
import i18n, { loadTranslations } from "./i18n/index.ts";
import { Widget } from "@seelen-ui/lib";

import "@shared/styles/reset.css";
import "@shared/styles/colors.css";

await declareDocumentAsLayeredHitbox();
await loadTranslations();
await initStore();

await Widget.getCurrent().init();

registerDocumentEvents();
disableAnimationsOnPerformanceMode();

createRoot(getRootContainer()).render(
  <Provider store={store}>
    <I18nextProvider i18n={i18n}>
      <App />
    </I18nextProvider>
  </Provider>,
);
