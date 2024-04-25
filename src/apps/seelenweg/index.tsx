import { getRootContainer } from '../utils';
import { wrapConsole } from '../utils/ConsoleWrapper';
import { ErrorBoundary } from './components/Error';
import { registerDocumentEvents, setWindowSize, updateHitbox } from './events';
import { SeelenWeg } from './modules/bar';
import { invoke } from '@tauri-apps/api/core';
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

async function Main() {
  wrapConsole();
  const container = getRootContainer();

  setWindowSize();
  registerDocumentEvents();

  await loadConstants();
  await loadStore();
  await registerStoreEvents();
  await invoke('enum_opened_apps');

  const WrappedRoot = () => {
    useEffect(() => {
      emitTo('seelenweg-hitbox', 'init').then(() => {
        updateHitbox();
        getCurrent().show();
      });
    }, []);

    return (
      <Provider store={store}>
        <ConfigProvider
          getPopupContainer={() => container}
          componentSize="small"
          theme={{
            algorithm: window.matchMedia('(prefers-color-scheme: dark)').matches
              ? theme.darkAlgorithm
              : theme.defaultAlgorithm,
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
