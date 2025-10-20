import { getRootContainer } from "@shared";
import { mount } from "svelte";
import App from "./app.svelte";
import { startThemingTool } from "libs/core/npm/esm/mod";
import { loadTranslations } from "./i18n/index.ts";

import "@shared/styles/reset.css";
import "@shared/styles/colors.css";

await loadTranslations();
await startThemingTool();

mount(App, {
  target: getRootContainer(),
});
