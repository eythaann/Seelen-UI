import { $system_colors } from '@shared/signals';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { ConfigProvider, theme } from 'antd';
import { useEffect } from 'react';

import { useDarkMode } from '../shared/styles';
import { ErrorBoundary } from './components/Error';
import { SeelenWeg } from './modules/bar';

async function onMount() {
  const view = getCurrentWebviewWindow();
  await view.show();
}

export function App() {
  const isDarkMode = useDarkMode();

  useEffect(() => {
    onMount();
    console.debug('Seelen Weg app mounted');
  }, []);

  return (
    <ConfigProvider
      componentSize="small"
      theme={{
        token: {
          colorPrimary: isDarkMode
            ? $system_colors.value.accent_light
            : $system_colors.value.accent_dark,
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
