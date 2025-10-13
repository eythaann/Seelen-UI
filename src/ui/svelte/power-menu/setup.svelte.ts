// import { PhysicalPosition, PhysicalSize } from "@tauri-apps/api/window";
import { Widget } from "@seelen-ui/lib";
// import { state } from "./state.svelte";

export async function setup() {
  let widget = Widget.getCurrent();
  let webview = widget.webview;
  // await updateSize();
  await webview.setAlwaysOnTop(true);
  await webview.show();
}

/* async function updateSize() {
  let widget = Widget.getCurrent();
  let webview = widget.webview;

  await webview.setPosition(new PhysicalPosition(state.desktopRect.left, state.desktopRect.top));
  await webview.setSize(
    new PhysicalSize({
      width: state.desktopRect.right - state.desktopRect.left,
      height: state.desktopRect.bottom - state.desktopRect.top,
    })
  );
} */
