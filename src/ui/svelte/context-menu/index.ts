import { mount } from "svelte";
import App from "./app.svelte";
import { Widget } from "@seelen-ui/lib";

import "@seelen-ui/lib/styles/reset.css";

const root = document.getElementById("root")!;

await Widget.self.init({
  autoSizeByContent: root,
});

// Listen for close requests from parent menu
Widget.self.webview.listen("contextmenu:close", () => {
  Widget.self.hide();
});

mount(App, {
  target: root,
});
