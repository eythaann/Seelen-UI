import { webviewInfo } from "./_tauri";
import { emitTo, listen } from "@tauri-apps/api/event";

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
