import { SeelenCommand, Widget } from "@seelen-ui/lib";
import { getRootContainer } from "@shared";
import { declareDocumentAsLayeredHitbox } from "@shared/layered";
import { disableAnimationsOnPerformanceMode } from "@shared/performance";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { createRoot } from "react-dom/client";
import { I18nextProvider } from "react-i18next";

import { App } from "./app.tsx";

import i18n, { loadTranslations } from "./i18n/index.ts";

import "@shared/styles/colors.css";
import "./styles/variables.css";
import "@shared/styles/reset.css";
import "./styles/global.css";

await declareDocumentAsLayeredHitbox();
await loadTranslations();
await Widget.getCurrent().init();

disableAnimationsOnPerformanceMode();

const container = getRootContainer();
createRoot(container).render(
  <I18nextProvider i18n={i18n}>
    <App />
  </I18nextProvider>,
);

getCurrentWebviewWindow().onDragDropEvent(async (e) => {
  if (e.payload.type === "drop") {
    for (const path of e.payload.paths) {
      await invoke(SeelenCommand.WegPinItem, { path });
    }
  }
});
