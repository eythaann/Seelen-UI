import { webviewInfo } from "./_tauri";
// import type { FocusedApp } from "@seelen-ui/lib/types";
import { emitTo, listen } from "@tauri-apps/api/event";

// for some reason the gc is not running for webview2 overlays as are our widgets
if (window.gc) {
  setInterval(() => {
    window.gc?.();
  }, 10000);
} else {
  console.warn("Garbage collector not available");
}

/* if (!window.__SLU_WIDGET.noMemoryLeakWorkaround) {
  // workaround for tauri/webview2 memory leak
  setInterval(async () => {
    const app = await _invoke<FocusedApp>("get_focused_app");
    // avoid reload the UI while playing as this can cause fps drops
    if (app.isFullscreened || app.exe?.endsWith("seelen-ui.exe")) {
      return;
    }

    console.trace("Reloading widget.");
    location.search = `r=${Date.now()}`; // add a query hash to force be a new page
  }, 60_000 * 10); // every 10 minutes
}*/

// important in case of unexpected crash like Out of Memory
listen<string>(
  "internal::liveness-ping",
  () => {
    emitTo(webviewInfo.rawLabel, "internal::liveness-pong");
  },
  {
    target: {
      kind: "WebviewWindow",
      label: webviewInfo.rawLabel,
    },
  },
);
