import { Widget } from "@seelen-ui/lib";
import { ErrorBoundary } from "../weg/components/Error/index.tsx";
import { ErrorFallback } from "./components/Error/index.tsx";
import { FancyToolbar } from "./modules/main/Toolbar.tsx";
import { useSignalEffect } from "@preact/signals";
import { $lastFocusedOnMonitor } from "./modules/shared/state/windows.ts";
import { $initialPositionSet } from "./modules/shared/state/mod.ts";
import { useEffect } from "preact/hooks";

export function App() {
  useEffect(() => {
    Widget.self.ready({ show: false });
  }, []);

  useSignalEffect(() => {
    const fullscreened = !!$lastFocusedOnMonitor.value?.isFullscreened;
    if (fullscreened) {
      Widget.self.hide();
    } else if ($initialPositionSet.value) {
      Widget.self.show();
    }
  });

  return (
    <ErrorBoundary fallback={<ErrorFallback />}>
      <FancyToolbar />
    </ErrorBoundary>
  );
}
