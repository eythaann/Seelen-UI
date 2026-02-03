import { getRootContainer } from "libs/ui/react/utils/index.ts";
import { createRoot } from "react-dom/client";
import { I18nextProvider } from "react-i18next";
import { HashRouter } from "react-router";
import { Widget } from "@seelen-ui/lib";
import { LogicalSize } from "@seelen-ui/lib/tauri";

import { App } from "./app.tsx";

import i18n, { loadTranslations } from "./i18n/index.ts";

import "@shared/styles/colors.css";
import "./styles/variables.css";
import "@shared/styles/reset.css";
import "./styles/global.css";
import "@shared/styles/RichText.css";

const { webview } = Widget.self;

await Promise.all([
  webview.setDecorations(false),
  webview.setSizeConstraints({ minWidth: 600, minHeight: 400 }),
  webview.setSize(new LogicalSize(800, 500)),
]);
await webview.center();

await Widget.self.init();
await Widget.self.show();
await Widget.self.focus();

Widget.self.onTrigger(() => {
  webview.unminimize();
  webview.setFocus();
});

await loadTranslations();

const container = getRootContainer();
createRoot(container).render(
  <I18nextProvider i18n={i18n}>
    <HashRouter>
      <App />
    </HashRouter>
  </I18nextProvider>,
);
