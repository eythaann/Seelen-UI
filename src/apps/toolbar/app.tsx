import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { ConfigProvider, theme } from 'antd';
import { useEffect } from 'react';
import { useSelector } from 'react-redux';

import { ToolBar } from './modules/main/infra';

import { Selectors } from './modules/shared/store/app';

import { ErrorBoundary } from '../seelenweg/components/Error';
import { useDarkMode } from '../shared/styles';
import { ErrorFallback } from './components/Error';

export function App() {
  const colors = useSelector(Selectors.colors);

  const isDarkMode = useDarkMode();

  useEffect(() => {
    getCurrentWebviewWindow().show();
  }, []);

  return (
    <ConfigProvider
      theme={{
        token: {
          colorPrimary: isDarkMode ? colors.accent_light : colors.accent_dark,
          motion: false,
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
