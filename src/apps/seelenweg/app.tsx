import { getRootContainer } from '../shared';
import { useDarkMode } from '../shared/styles';
import { ErrorBoundary } from './components/Error';
import { updateHitbox } from './events';
import { SeelenWeg } from './modules/bar';
import { emit, emitTo } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { ConfigProvider, theme } from 'antd';
import { useEffect } from 'react';
import { useSelector } from 'react-redux';

import { Selectors } from './modules/shared/store/app';

async function onMount() {
  let view = getCurrentWebviewWindow();
  updateHitbox();
  await emitTo(view.label.replace('/', '-hitbox/'), 'init');
  await view.show();
  await view.emitTo(view.label, 'complete-setup');
  await emit('register-colors-events');
}
export function App() {
  const isDarkMode = useDarkMode();
  const colors = useSelector(Selectors.colors);

  useEffect(() => {
    onMount();
  }, []);

  return (
    <ConfigProvider
      getPopupContainer={getRootContainer}
      componentSize="small"
      theme={{
        token: {
          colorPrimary: isDarkMode ? colors.accent_light : colors.accent_dark,
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
