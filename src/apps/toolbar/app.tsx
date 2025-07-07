import { $system_colors } from '@shared/signals';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { ConfigProvider, theme } from 'antd';
import { useEffect } from 'react';

import { ToolBar } from './modules/main/infra';

import { ErrorBoundary } from '../seelenweg/components/Error';
import { useDarkMode } from '../shared/styles';
import { ErrorFallback } from './components/Error';

export function App() {
  const isDarkMode = useDarkMode();

  useEffect(() => {
    getCurrentWebviewWindow().show();
  }, []);

  return (
    <ConfigProvider
      theme={{
        token: {
          colorPrimary: isDarkMode
            ? $system_colors.value.accent_light
            : $system_colors.value.accent_dark,
        },
        components: {
          Calendar: {
            fullBg: 'transparent',
            fullPanelBg: 'transparent',
            itemActiveBg: 'transparent',
          },
        },
        algorithm: isDarkMode ? theme.darkAlgorithm : theme.defaultAlgorithm,
      }}
    >
      <ErrorBoundary fallback={<ErrorFallback />}>
        <ToolBar />
      </ErrorBoundary>
    </ConfigProvider>
  );
}
