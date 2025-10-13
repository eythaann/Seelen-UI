import { getRootContainer } from "@shared";
import { mount } from "svelte";
import App from "./app.svelte";
import { startThemingTool } from "libs/core/npm/esm/mod";

import "@shared/styles/reset.css";
import "@shared/styles/colors.css";

await startThemingTool();

mount(App, {
  target: getRootContainer(),
});
