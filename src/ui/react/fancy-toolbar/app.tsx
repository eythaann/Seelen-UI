import { Widget } from "@seelen-ui/lib";
import { ErrorBoundary } from "../weg/components/Error/index.tsx";
import { ErrorFallback } from "./components/Error/index.tsx";
import { FancyToolbar } from "./modules/main/Toolbar.tsx";
import { computed, useSignalEffect } from "@preact/signals";
import { $focused, $top_interactable_window, $widget_statuses } from "./modules/shared/state/windows.ts";
import { useEffect } from "preact/hooks";

const topWindowIsFullscreen = computed(() => $top_interactable_window.value?.isFullscreen);
const focusedIsAppsMenu = computed(() =>
  $widget_statuses.value.some(
    (w) => w.widgetId === "@seelen/apps-menu" && w.webviewWindowId === $focused.value.hwnd,
  )
);
const alwaysOnTop = computed(() => !topWindowIsFullscreen.value || focusedIsAppsMenu.value);

export function App() {
  useEffect(() => {
    Widget.self.ready();
  }, []);

  useSignalEffect(() => {
    if (alwaysOnTop.value) {
      Widget.self.window.setAlwaysOnBottom(false);
      Widget.self.window.setAlwaysOnTop(true);
    } else {
      Widget.self.window.setAlwaysOnTop(false);
      Widget.self.window.setAlwaysOnBottom(true);
    }
  });

  return (
    <ErrorBoundary fallback={<ErrorFallback />}>
      <FancyToolbar />
    </ErrorBoundary>
  );
}
