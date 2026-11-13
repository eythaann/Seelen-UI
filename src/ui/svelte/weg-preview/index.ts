import { mount } from "svelte";
import App from "./App.svelte";
import { Widget } from "@seelen-ui/lib";

import "@seelen-ui/lib/styles/reset.css";

const root = document.getElementById("root")!;

await Widget.self.init({
  autoSizeByContent: root,
  hideOnFocusLoss: false,
  focusOnTrigger: false,
  closeOnHide: false,
});

mount(App, {
  target: root,
});
