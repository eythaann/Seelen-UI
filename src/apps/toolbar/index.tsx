import { getRootContainer } from '../shared';
import { wrapConsole } from '../shared/ConsoleWrapper';
import { registerDocumentEvents } from './events';
import i18n, { loadTranslations } from './i18n';
import { createRoot } from 'react-dom/client';
import { I18nextProvider } from 'react-i18next';
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
  await registerDocumentEvents();

  const container = getRootContainer();

  await loadConstants();
  await registerStoreEvents();
  await loadStore();
  await loadTranslations();

  createRoot(container).render(
    <Provider store={store}>
      <I18nextProvider i18n={i18n}>
        <App />
      </I18nextProvider>
    </Provider>,
  );
}

Main();
