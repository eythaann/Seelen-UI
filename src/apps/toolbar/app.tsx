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
  const structure = useSelector(Selectors.placeholder);
  const colors = useSelector(Selectors.colors);

  const isDarkMode = useDarkMode();

  useEffect(() => {
    getCurrentWebviewWindow().show();
  }, []);

  if (!structure) {
    return <ErrorFallback msg="NO PLACEHOLDER FOUND. PLEASE TRY REINSTALL THE APP" />;
  }

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
        <ToolBar structure={structure} />
      </ErrorBoundary>
    </ConfigProvider>
  );
}
