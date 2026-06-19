import { getRootContainer } from "libs/ui/react/utils/index.ts";
import { mount } from "svelte";
import App from "./App.svelte";
import { Widget } from "@seelen-ui/lib";

import "./styles/variables.css";
import "@seelen-ui/lib/styles/reset.css";
import "./styles/global.css";

await Widget.getCurrent().init();

mount(App, {
  target: getRootContainer(),
});
