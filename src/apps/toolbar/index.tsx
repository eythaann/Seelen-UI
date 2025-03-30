import { declareDocumentAsLayeredHitbox, disableWebviewShortcutsAndContextMenu } from '@shared/setup';
import { createRoot } from 'react-dom/client';
import { I18nextProvider } from 'react-i18next';
import { Provider } from 'react-redux';

import { registerStoreEvents, store } from './modules/shared/store/infra';

import { App } from './app';

import { getRootContainer } from '../shared';
import i18n, { loadTranslations } from './i18n';

import '../shared/styles/colors.css';
import './styles/variables.css';
import '../shared/styles/reset.css';
import './styles/global.css';

disableWebviewShortcutsAndContextMenu();
await declareDocumentAsLayeredHitbox();
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
