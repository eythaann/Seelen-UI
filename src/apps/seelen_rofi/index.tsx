import { createRoot } from 'react-dom/client';
import { I18nextProvider } from 'react-i18next';
import { Provider } from 'react-redux';
import { declareDocumentAsLayeredHitbox } from 'seelen-core';

import { initStore, store } from './modules/shared/store/infra';

import { getRootContainer } from '../shared';
import { wrapConsole } from '../shared/ConsoleWrapper';
import { App } from './App';
import { registerDocumentEvents } from './events';
import i18n from './i18n';

import './styles/reset.css';
import './styles/colors.css';

async function Main() {
  wrapConsole();
  await declareDocumentAsLayeredHitbox();
  await initStore();
  registerDocumentEvents();

  createRoot(getRootContainer()).render(
    <Provider store={store}>
      <I18nextProvider i18n={i18n}>
        <App />
      </I18nextProvider>
    </Provider>,
  );
}

Main();
