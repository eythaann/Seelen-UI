import { ErrorBoundary } from '../seelenweg/components/Error';
import { getRootContainer } from '../shared';
import { ErrorFallback } from './components/Error';
import { emitTo } from '@tauri-apps/api/event';
import { getCurrent } from '@tauri-apps/api/webviewWindow';
import { ConfigProvider, theme } from 'antd';
import { useEffect } from 'react';
import { useSelector } from 'react-redux';

import { ToolBar } from './modules/main/infra';

import { Selectors } from './modules/shared/store/app';

async function onMount() {
  let view = getCurrent();
  await emitTo(view.label.replace('/', '-hitbox/'), 'init');
  await view.show();
}

export function App() {
  const structure = useSelector(Selectors.placeholder);
  const accentColor = useSelector(Selectors.accentColor);

  useEffect(() => {
    onMount();
  }, []);

  if (!structure) {
    return <ErrorFallback msg="NO PLACEHOLDER FOUND. PLEASE TRY REINSTALL THE APP" />;
  }

  return (
    <ConfigProvider
      getPopupContainer={() => getRootContainer()}
      theme={{
        token: {
          colorPrimary: accentColor,
        },
        algorithm: window.matchMedia('(prefers-color-scheme: dark)').matches
          ? theme.darkAlgorithm
          : theme.defaultAlgorithm,
      }}
    >
      <ErrorBoundary fallback={<ErrorFallback />}>
        <ToolBar structure={structure} />
      </ErrorBoundary>
    </ConfigProvider>
  );
}
