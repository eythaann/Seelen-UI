import { createRoot } from 'react-dom/client';
import { I18nextProvider } from 'react-i18next';
import { Provider } from 'react-redux';
import { declareDocumentAsLayeredHitbox } from 'seelen-core';

import { loadStore, registerStoreEvents, store } from './modules/shared/store/infra';
import { loadConstants } from './modules/shared/utils/infra';

import { App } from './app';

import { getRootContainer } from '../shared';
import { wrapConsole } from '../shared/ConsoleWrapper';
import i18n, { loadTranslations } from './i18n';

import './styles/colors.css';
import './styles/variables.css';
import './styles/reset.css';
import './styles/global.css';

async function Main() {
  wrapConsole();
  await declareDocumentAsLayeredHitbox();
  await loadConstants();
  await loadStore();
  await registerStoreEvents();
  await loadTranslations();

  const container = getRootContainer();
  createRoot(container).render(
    <Provider store={store}>
      <I18nextProvider i18n={i18n}>
        <App />
      </I18nextProvider>
    </Provider>,
  );
}

Main();
