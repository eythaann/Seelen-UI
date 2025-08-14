import { getRootContainer } from '@shared';
import { createRoot } from 'react-dom/client';
import { Provider } from 'react-redux';

import { store } from './modules/shared/store/infra';

import { App } from './app';

import '@shared/styles/colors.css';
import '@shared/styles/reset.css';
import './styles/global.css';

async function Main() {
  const container = getRootContainer();
  createRoot(container).render(
    <Provider store={store}>
      <App />
    </Provider>,
  );
}

Main();
