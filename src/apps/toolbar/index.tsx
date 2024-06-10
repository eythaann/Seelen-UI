import { getRootContainer } from '../utils';
import { wrapConsole } from '../utils/ConsoleWrapper';
import { registerDocumentEvents } from './events';
import { ConfigProvider, theme } from 'antd';
import { createRoot } from 'react-dom/client';
import { Provider } from 'react-redux';

import { loadStore, registerStoreEvents, store } from './modules/shared/store/infra';
import { loadConstants } from './modules/shared/utils/infra';

import { App } from './app';

import './styles/colors.css';
import './styles/variables.css';
import './styles/reset.css';
import './styles/global.css';

async function Main() {
  wrapConsole();
  const container = getRootContainer();

  await loadConstants();
  await registerStoreEvents();
  await loadStore();
  registerDocumentEvents();

  window.TOOLBAR_MODULES = {} as any;

  createRoot(container).render(
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
        <App />
      </ConfigProvider>
    </Provider>,
  );
}

Main();
