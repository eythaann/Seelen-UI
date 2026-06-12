import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import { ZOrder } from "@seelen-ui/lib/types";

import { ErrorBoundary } from "./components/Error/index.tsx";
import { SeelenWeg } from "./modules/bar/index.tsx";
import { computed, effect } from "@preact/signals";
import { $focused, $top_interactable_window, $widget_statuses } from "./modules/shared/state/windows.ts";

import { useEffect } from "preact/hooks";
import { debounce } from "lodash";

const startMenuExes = ["SearchHost.exe", "StartMenuExperienceHost.exe"];

const topWindowIsFullscreen = computed(() => $top_interactable_window.value?.isFullscreen);
const focusedIsAppsMenu = computed(
  () =>
    startMenuExes.some((program) => ($focused.value.exe || "").endsWith(program)) ||
    $widget_statuses.value.some(
      (w) => w.widgetId === "@seelen/apps-menu" && w.webviewWindowId === $focused.value.hwnd,
    ),
);
const alwaysOnTop = computed(() => !topWindowIsFullscreen.value || focusedIsAppsMenu.value);

const setAlwaysOnTop = debounce((alwaysOnTop: boolean) => {
  if (alwaysOnTop) {
    invoke(SeelenCommand.SetSelfZOrder, { zOrder: ZOrder.TopMost });
  } else {
    invoke(SeelenCommand.SetSelfZOrder, { zOrder: ZOrder.Bottom });
  }
}, 200);

effect(() => {
  setAlwaysOnTop(alwaysOnTop.value);
});

export function App() {
  useEffect(() => {
    Widget.self.ready();
  }, []);

  return (
    <ErrorBoundary fallback={<div>Something went wrong</div>}>
      <SeelenWeg />
    </ErrorBoundary>
  );
}
