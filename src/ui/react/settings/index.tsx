import { getRootContainer } from "@shared";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { createRoot } from "react-dom/client";
import { I18nextProvider } from "react-i18next";
import { Provider } from "react-redux";
import { HashRouter } from "react-router";
import { Widget } from "@seelen-ui/lib";

import { LoadSettingsToStore, registerStoreEvents, store } from "./modules/shared/store/infra.ts";

import { App } from "./app.tsx";

import i18n, { loadTranslations } from "./i18n/index.ts";

import "@shared/styles/colors.css";
import "./styles/variables.css";
import "@shared/styles/reset.css";
import "./styles/global.css";

getCurrentWebviewWindow().show();

await LoadSettingsToStore();
await registerStoreEvents();
await loadTranslations();

await Widget.getCurrent().init();

const container = getRootContainer();
createRoot(container).render(
  <Provider store={store}>
    <I18nextProvider i18n={i18n}>
      <HashRouter>
        <App />
      </HashRouter>
    </I18nextProvider>
  </Provider>,
);
