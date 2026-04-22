import { $system_colors } from "libs/ui/react/utils/signals.ts";
import { useDarkMode } from "libs/ui/react/utils/styling.ts";
import { ConfigProvider, theme } from "antd";
import { Widget } from "@seelen-ui/lib";
import { ErrorBoundary } from "../weg/components/Error/index.tsx";
import { ErrorFallback } from "./components/Error/index.tsx";
import { FancyToolbar } from "./modules/main/Toolbar.tsx";
import { useSignalEffect } from "@preact/signals";
import { $lastFocusedOnMonitor } from "./modules/shared/state/windows.ts";
import { useEffect } from "preact/hooks";

export function App() {
  const isDarkMode = useDarkMode();

  useEffect(() => {
    Widget.self.ready({ show: false });
  }, []);

  useSignalEffect(() => {
    const fullscreened = !!$lastFocusedOnMonitor.value?.isFullscreened;
    if (fullscreened) {
      Widget.self.hide();
    } else {
      Widget.self.show();
    }
  });

  return (
    <ConfigProvider
      theme={{
        token: {
          colorPrimary: isDarkMode ? $system_colors.value.accent_light : $system_colors.value.accent_dark,
        },
        components: {
          Calendar: {
            fullBg: "transparent",
            fullPanelBg: "transparent",
            itemActiveBg: "transparent",
          },
        },
        algorithm: isDarkMode ? theme.darkAlgorithm : theme.defaultAlgorithm,
      }}
    >
      <ErrorBoundary fallback={<ErrorFallback />}>
        <FancyToolbar />
      </ErrorBoundary>
    </ConfigProvider>
  );
}
