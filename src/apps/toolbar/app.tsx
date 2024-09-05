import { ErrorBoundary } from '../seelenweg/components/Error';
import { getRootContainer } from '../shared';
import { useDarkMode } from '../shared/styles';
import { ErrorFallback } from './components/Error';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { ConfigProvider, theme } from 'antd';
import { useEffect } from 'react';
import { useSelector } from 'react-redux';

import { ToolBar } from './modules/main/infra';

import { Selectors } from './modules/shared/store/app';

export function App() {
  const version = useSelector(Selectors.version);

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
      getPopupContainer={() => getRootContainer()}
      theme={{
        token: {
          colorPrimary: isDarkMode ? colors.accent_light : colors.accent_dark,
        },
        algorithm: isDarkMode ? theme.darkAlgorithm : theme.defaultAlgorithm,
      }}
    >
      <ErrorBoundary key={version} fallback={<ErrorFallback />}>
        <ToolBar key={version} structure={structure} />
      </ErrorBoundary>
    </ConfigProvider>
  );
}
