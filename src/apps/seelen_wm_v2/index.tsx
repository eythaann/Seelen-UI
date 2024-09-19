import { createRoot } from 'react-dom/client';
import { Provider } from 'react-redux';

import { loadStore, store } from './modules/shared/store/infra';

import { App } from './app';

import { getRootContainer } from '../shared';
import { wrapConsole } from '../shared/ConsoleWrapper';

import './styles/colors.css';
import './styles/reset.css';
import './styles/global.css';

async function Main() {
  wrapConsole();
  await loadStore();

  const container = getRootContainer();
  createRoot(container).render(
    <Provider store={store}>
      <App />
    </Provider>,
  );
}

Main();
