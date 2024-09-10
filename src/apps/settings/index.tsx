import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { createRoot } from 'react-dom/client';
import { I18nextProvider } from 'react-i18next';
import { Provider } from 'react-redux';

import { LoadSettingsToStore, registerStoreEvents, store } from './modules/shared/store/infra';

import { App } from './app';

import { getRootContainer } from '../shared';
import { wrapConsole } from '../shared/ConsoleWrapper';
import i18n, { loadTranslations } from './i18n';

import './styles/colors.css';
import './styles/variables.css';
import './styles/reset.css';
import './styles/global.css';

(async function main() {
  wrapConsole();
  getCurrentWebviewWindow().show();
  const container = getRootContainer();

  await LoadSettingsToStore();
  await registerStoreEvents();
  await loadTranslations();

  createRoot(container).render(
    <Provider store={store}>
      <I18nextProvider i18n={i18n}>
        <App />
      </I18nextProvider>
    </Provider>,
  );
})();
