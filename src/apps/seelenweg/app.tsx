import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { ConfigProvider, theme } from 'antd';
import { useEffect } from 'react';
import { useSelector } from 'react-redux';

import { Selectors } from './modules/shared/store/app';

import { useDarkMode } from '../shared/styles';
import { ErrorBoundary } from './components/Error';
import { SeelenWeg } from './modules/bar';

async function onMount() {
  const view = getCurrentWebviewWindow();
  await view.show();
}

export function App() {
  const isDarkMode = useDarkMode();
  const colors = useSelector(Selectors.colors);

  useEffect(() => {
    onMount();
    console.debug('Seelen Weg app mounted');
  }, []);

  return (
    <ConfigProvider
      componentSize="small"
      theme={{
        token: {
          colorPrimary: isDarkMode ? colors.accent_light : colors.accent_dark,
          motion: false,
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
