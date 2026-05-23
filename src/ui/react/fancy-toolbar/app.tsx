import { Widget } from "@seelen-ui/lib";
import { ErrorBoundary } from "../weg/components/Error/index.tsx";
import { ErrorFallback } from "./components/Error/index.tsx";
import { FancyToolbar } from "./modules/main/Toolbar.tsx";
import { useSignalEffect } from "@preact/signals";
import { $isSomeFullscreenOnMonitor } from "./modules/shared/state/windows.ts";
import { $initialPositionSet } from "./modules/shared/state/mod.ts";
import { useEffect } from "preact/hooks";

export function App() {
  useEffect(() => {
    Widget.self.ready();
  }, []);

  useSignalEffect(() => {
    const fullscreened = $isSomeFullscreenOnMonitor.value;
    if (fullscreened) {
      Widget.self.window.setAlwaysOnTop(false);
      Widget.self.window.setAlwaysOnBottom(true);
    } else if ($initialPositionSet.value) {
      Widget.self.window.setAlwaysOnBottom(false);
      Widget.self.window.setAlwaysOnTop(true);
    }
  });

  return (
    <ErrorBoundary fallback={<ErrorFallback />}>
      <FancyToolbar />
    </ErrorBoundary>
  );
}
