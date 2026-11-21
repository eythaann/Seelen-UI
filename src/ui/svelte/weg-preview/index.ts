import { mount } from "svelte";
import App from "./App.svelte";
import { Widget } from "@seelen-ui/lib";

import "@seelen-ui/lib/styles/reset.css";

const root = document.getElementById("root")!;

await Widget.self.init({
  autoSizeByContent: root,
  // Re-triggered with new content (another dock item) while visible.
  toggleOnTrigger: false,
  // closeOnHide: false,
});

mount(App, {
  target: root,
});
