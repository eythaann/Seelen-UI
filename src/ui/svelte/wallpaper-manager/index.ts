import { mount } from "svelte";
import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import App from "./app.svelte";
import { loadTranslations } from "./i18n/index.ts";

import "@seelen-ui/lib/styles/reset.css";

await loadTranslations();
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

const root = document.getElementById("root")!;
mount(App, { target: root });
