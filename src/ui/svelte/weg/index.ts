import { getRootContainer } from "libs/ui/react/utils/index.ts";
import { mount } from "svelte";
import App from "./App.svelte";
import { loadTranslations } from "./i18n/index.ts";
import { SeelenCommand, Widget } from "@seelen-ui/lib";
import { invoke } from "@tauri-apps/api/core";

import "./styles/variables.css";
import "@seelen-ui/lib/styles/reset.css";
import "./styles/global.css";

await loadTranslations();
await Widget.getCurrent().init();

mount(App, {
  target: getRootContainer(),
});

Widget.self.window.onDragDropEvent(async (e) => {
  if (e.payload.type === "drop") {
    for (const path of e.payload.paths) {
      await invoke(SeelenCommand.WegPinItem, { path });
    }
  }
});
