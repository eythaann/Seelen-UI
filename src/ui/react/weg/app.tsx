import { ErrorBoundary } from "./components/Error/index.tsx";
import { SeelenWeg } from "./modules/bar/index.tsx";
import { useSignalEffect } from "@preact/signals";
import { $top_interactable_window } from "./modules/shared/state/windows.ts";
import { Widget } from "@seelen-ui/lib";
import { $initialPositionSet } from "./modules/shared/state/settings.ts";
import { useEffect } from "preact/hooks";

export function App() {
  useEffect(() => {
    Widget.self.ready();
  }, []);

  useSignalEffect(() => {
    const fullscreened = $top_interactable_window.value?.isFullscreen;
    if (fullscreened) {
      Widget.self.window.setAlwaysOnTop(false);
      Widget.self.window.setAlwaysOnBottom(true);
    } else if ($initialPositionSet.value) {
      Widget.self.window.setAlwaysOnBottom(false);
      Widget.self.window.setAlwaysOnTop(true);
    }
  });

  return (
    <ErrorBoundary fallback={<div>Something went wrong</div>}>
      <SeelenWeg />
    </ErrorBoundary>
  );
}
