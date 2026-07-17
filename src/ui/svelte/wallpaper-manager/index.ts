import { mount } from "svelte";
import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import App from "./app.svelte";
import "@seelen-ui/lib/styles/reset.css";

await Widget.self.init({ saveAndRestoreLastRect: false });
await Widget.self.window.setResizable(false);
// Must be called before setIgnoreCursorEvents. tao's apply_diff (dispatched async to the main
// thread) calls SetWindowLongW(GWL_STYLE, ...) without WS_CHILD (tao doesn't know the window
// was reparented to WorkerW via SetParent). This puts the window in an inconsistent state:
// it has a parent but no WS_CHILD style. WorkerW often has WS_EX_NOREDIRECTIONBITMAP (Win11
// desktop layer), which makes WS_EX_LAYERED unsupported on its children per MSDN — those
// styles are silently dropped. Additionally WS_EX_WINDOWEDGE is restored — causing
// the window to disappear from WorkerW when SetParent is called with WINDOWEDGE set.
// Attaching before setIgnoreCursorEvents ensures SetParent runs while the window is still a
// normal top-level window and no conflicting apply_diff is pending on the main thread.
await invoke(SeelenCommand.SetAsWallpaper);
// await Widget.self.window.setIgnoreCursorEvents(true); btw this does nothing so better not call it.

// Polling instead of Widget.self.normalizeDevicePixelRatio() (onScaleChanged event): this
// window is reparented to WorkerW via SetParent, and onScaleChanged doesn't fire reliably
// there, so we poll devicePixelRatio instead.
let oldDPR = globalThis.devicePixelRatio;
async function lookupDPI() {
  if (globalThis.devicePixelRatio !== 1) {
    // when zoom was set dpr changed, so in case of change this is accomulative unit
    oldDPR = oldDPR * globalThis.devicePixelRatio;
    await Widget.self.webview.setZoom(1 / oldDPR);
  }
  setTimeout(lookupDPI, 500);
}
await lookupDPI();

const root = document.getElementById("root")!;
mount(App, { target: root });
