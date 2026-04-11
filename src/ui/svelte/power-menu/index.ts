import { getRootContainer } from "libs/ui/react/utils/index.ts";
import { mount } from "svelte";
import App from "./app.svelte";
import { loadTranslations } from "./i18n/index.ts";
import { Widget } from "@seelen-ui/lib";

import "@shared/styles/reset.css";
import "@shared/styles/colors.css";
import { debounce } from "lodash";

await loadTranslations();

const widget = Widget.getCurrent();
widget.onTrigger(async () => {
  await widget.show();
  await widget.focus();
});

const hide = debounce(() => {
  widget.hide(true);
}, 100);

widget.window.onFocusChanged(({ payload: focused }) => {
  if (focused) {
    hide.cancel();
  } else {
    hide();
  }
});

await widget.init({ normalizeDevicePixelRatio: true });
await widget.window.setResizable(false);

mount(App, {
  target: getRootContainer(),
});
