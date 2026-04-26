import { ErrorBoundary } from "./components/Error/index.tsx";
import { SeelenWeg } from "./modules/bar/index.tsx";
import { useSignalEffect } from "@preact/signals";
import { $lastFocusedOnMonitor } from "./modules/shared/state/windows.ts";
import { Widget } from "@seelen-ui/lib";

export function App() {
  useSignalEffect(() => {
    const fullscreened = !!$lastFocusedOnMonitor.value?.isFullscreened;
    if (fullscreened) {
      Widget.self.hide();
    } else {
      Widget.self.show();
    }
  });

  return (
    <ErrorBoundary fallback={<div>Something went wrong</div>}>
      <SeelenWeg />
    </ErrorBoundary>
  );
}
