import { getRootContainer } from '@shared';
import { removeDefaultWebviewActions } from '@shared/setup';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { createRoot } from 'react-dom/client';
import { I18nextProvider } from 'react-i18next';
import { Provider } from 'react-redux';
import { HashRouter } from 'react-router';

import { LoadSettingsToStore, registerStoreEvents, store } from './modules/shared/store/infra';

import { App } from './app';

import i18n, { loadTranslations } from './i18n';

import '@shared/styles/colors.css';
import './styles/variables.css';
import '@shared/styles/reset.css';
import './styles/global.css';

removeDefaultWebviewActions();
getCurrentWebviewWindow().show();

await LoadSettingsToStore();
await registerStoreEvents();
await loadTranslations();

const container = getRootContainer();
createRoot(container).render(
  <Provider store={store}>
    <I18nextProvider i18n={i18n}>
      <HashRouter>
        <App />
      </HashRouter>
    </I18nextProvider>
  </Provider>,
);
