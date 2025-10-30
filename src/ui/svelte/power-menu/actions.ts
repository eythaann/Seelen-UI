import { PhysicalPosition, PhysicalSize } from "@tauri-apps/api/window";
import { Widget } from "@seelen-ui/lib";
import type { State } from "./state.svelte.ts";
import { listen } from "@tauri-apps/api/event";

let widget = Widget.getCurrent();
let webview = widget.webview;

export async function setup(state: State) {
  webview.setResizable(false);

  listen("power-menu::show", () => {
    webview.show();
  });

  webview.onFocusChanged((e) => {
    if (!e.payload) {
      webview.hide();
    }
  });

  widget.onTrigger(() => {
    webview.show();
  });

  await updateSize(state);
  await widget.init();
}

async function updateSize(state: State) {
  await webview.setPosition(new PhysicalPosition(state.desktopRect.left, state.desktopRect.top));
  await webview.setSize(
    new PhysicalSize({
      width: state.desktopRect.right - state.desktopRect.left,
      height: state.desktopRect.bottom - state.desktopRect.top,
    }),
  );
}
