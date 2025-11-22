import { $system_colors } from "@shared/signals";
import { useDarkMode } from "@shared/styles";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { ConfigProvider, theme } from "antd";

import { ErrorBoundary } from "./components/Error/index.tsx";
import { SeelenWeg } from "./modules/bar/index.tsx";
import { useSignalEffect } from "@preact/signals";
import { $lastFocusedOnMonitor } from "./modules/shared/state/windows.ts";

const webview = getCurrentWebviewWindow();

export function App() {
  const isDarkMode = useDarkMode();

  useSignalEffect(() => {
    const fullscreened = !!$lastFocusedOnMonitor.value?.isFullscreened;
    if (fullscreened) {
      webview.hide();
    } else {
      webview.show();
    }
  });

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
