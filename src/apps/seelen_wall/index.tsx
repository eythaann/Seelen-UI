import { createRoot } from 'react-dom/client';
import { Provider } from 'react-redux';

import { initStore, store } from './modules/shared/store/infra';

import { App } from './app';

import { getRootContainer } from '../shared';

import '../shared/styles/colors.css';
import '../shared/styles/reset.css';
import './styles/global.css';

async function Main() {
  await initStore();

  const container = getRootContainer();
  createRoot(container).render(
    <Provider store={store}>
      <App />
    </Provider>,
  );
}

Main();
