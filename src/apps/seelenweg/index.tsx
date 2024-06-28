import { getRootContainer } from '../shared';
import { wrapConsole } from '../shared/ConsoleWrapper';
import { useDarkMode } from '../shared/styles';
import { ErrorBoundary } from './components/Error';
import { registerDocumentEvents, updateHitbox } from './events';
import { SeelenWeg } from './modules/bar';
import { emitTo } from '@tauri-apps/api/event';
import { getCurrent } from '@tauri-apps/api/webviewWindow';
import { ConfigProvider, theme } from 'antd';
import { useEffect } from 'react';
import { createRoot } from 'react-dom/client';
import { Provider } from 'react-redux';

import { loadStore, registerStoreEvents, store } from './modules/shared/store/infra';
import { loadConstants } from './modules/shared/utils/infra';

import './styles/colors.css';
import './styles/variables.css';
import './styles/reset.css';
import './styles/global.css';

async function onMount() {
  let view = getCurrent();
  updateHitbox();
  await emitTo(view.label.replace('/', '-hitbox/'), 'init');
  await view.show();
  await view.emitTo(view.label, 'complete-setup');
}

async function Main() {
  wrapConsole();
  const container = getRootContainer();

  registerDocumentEvents();

  await loadConstants();
  await loadStore();
  await registerStoreEvents();

  const WrappedRoot = () => {
    const isDarkMode = useDarkMode();

    useEffect(() => {
      onMount();
    }, []);

    return (
      <Provider store={store}>
        <ConfigProvider
          getPopupContainer={() => container}
          componentSize="small"
          theme={{
            algorithm: isDarkMode ? theme.darkAlgorithm : theme.defaultAlgorithm,
          }}
        >
          <ErrorBoundary fallback={<div>Something went wrong</div>}>
            <SeelenWeg />
          </ErrorBoundary>
        </ConfigProvider>
      </Provider>
    );
  };

  createRoot(container).render(<WrappedRoot />);
}

Main();
