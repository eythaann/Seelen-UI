import { ErrorBoundary } from "./components/Error/index.tsx";
import { SeelenWeg } from "./modules/bar/index.tsx";
import { useSignalEffect } from "@preact/signals";
import { $lastFocusedOnMonitor } from "./modules/shared/state/windows.ts";
import { Widget } from "@seelen-ui/lib";
import { $initialPositionSet } from "./modules/shared/state/settings.ts";
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
    <ErrorBoundary fallback={<div>Something went wrong</div>}>
      <SeelenWeg />
    </ErrorBoundary>
  );
}
