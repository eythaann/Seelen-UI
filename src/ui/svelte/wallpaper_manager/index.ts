import { mount } from "svelte";
import { Widget } from "@seelen-ui/lib";
import App from "./app.svelte";
import { loadTranslations } from "./i18n/index.ts";

import "@seelen-ui/lib/styles/reset.css";

await Widget.getCurrent().init();
await loadTranslations();

const root = document.getElementById("root")!;
mount(App, { target: root });
