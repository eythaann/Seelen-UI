import { $system_colors } from "@shared/signals";
import { useDarkMode } from "@shared/styles";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { ConfigProvider, theme } from "antd";
import { useEffect } from "react";

import { ErrorBoundary } from "./components/Error/index.tsx";
import { SeelenWeg } from "./modules/bar/index.tsx";

export function App() {
  const isDarkMode = useDarkMode();

  useEffect(() => {
    const view = getCurrentWebviewWindow();
    view.show();
  }, []);

  return (
    <ConfigProvider
      componentSize="small"
      theme={{
        token: {
          colorPrimary: isDarkMode ? $system_colors.value.accent_light : $system_colors.value.accent_dark,
        },
        algorithm: isDarkMode ? theme.darkAlgorithm : theme.defaultAlgorithm,
      }}
    >
      <ErrorBoundary fallback={<div>Something went wrong</div>}>
        <SeelenWeg />
      </ErrorBoundary>
    </ConfigProvider>
  );
}
