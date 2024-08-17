import { ErrorBoundary } from '../seelenweg/components/Error';
import { getRootContainer } from '../shared';
import { useDarkMode } from '../shared/styles';
import { ErrorFallback } from './components/Error';
import { emit, emitTo } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { ConfigProvider, theme } from 'antd';
import { useEffect } from 'react';
import { useSelector } from 'react-redux';

import { ToolBar } from './modules/main/infra';

import { Selectors } from './modules/shared/store/app';

async function onMount() {
  let view = getCurrentWebviewWindow();
  await emitTo(view.label.replace('/', '-hitbox/'), 'init');
  await view.show();
  await emit('register-colors-events');
}

export function App() {
  const version = useSelector(Selectors.version);

  const structure = useSelector(Selectors.placeholder);
  const colors = useSelector(Selectors.colors);

  const isDarkMode = useDarkMode();

  useEffect(() => {
    onMount().catch(console.error);
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
