import { mount } from "svelte";
import { Widget } from "@seelen-ui/lib";
import App from "./app.svelte";
import { loadTranslations } from "./i18n/index.ts";

import "@shared/styles/colors.css";
import "@shared/styles/reset.css";

await Widget.getCurrent().init();
Widget.self.window.setFocusable(false);

await loadTranslations();

const root = document.getElementById("root")!;
mount(App, { target: root });
