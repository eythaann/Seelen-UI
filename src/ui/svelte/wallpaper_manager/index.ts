import { mount } from "svelte";
import { Widget } from "@seelen-ui/lib";
import { disableAnimationsOnPerformanceMode } from "libs/ui/react/utils/performance.ts";
import App from "./app.svelte";
import { loadTranslations } from "./i18n/index.ts";

import "@shared/styles/colors.css";
import "@shared/styles/reset.css";

await Widget.getCurrent().init();
Widget.self.webview.setFocusable(false);

await loadTranslations();
disableAnimationsOnPerformanceMode();

const root = document.getElementById("root")!;
mount(App, { target: root });
