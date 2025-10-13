import { $system_colors } from "@shared/signals";
import { useDarkMode } from "@shared/styles";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { ConfigProvider, theme } from "antd";
import { useEffect } from "react";

import { ErrorBoundary } from "../weg/components/Error/index.tsx";
import { ErrorFallback } from "./components/Error/index.tsx";
import { FancyToolbar } from "./modules/main/Toolbar.tsx";

export function App() {
  const isDarkMode = useDarkMode();

  useEffect(() => {
    getCurrentWebviewWindow().show();
  }, []);

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
