import { getRootContainer } from '@shared';
import { declareDocumentAsLayeredHitbox } from '@shared/layered';
import { removeDefaultWebviewActions } from '@shared/setup';
import { createRoot } from 'react-dom/client';
import { Provider } from 'react-redux';

import { loadStore, store } from './modules/shared/store/infra';

import { App } from './app';

import '@shared/styles/colors.css';
import '@shared/styles/reset.css';
import './styles/global.css';

removeDefaultWebviewActions();
await declareDocumentAsLayeredHitbox((e) => e.getAttribute('data-allow-mouse-events') === 'true');
await loadStore();

const container = getRootContainer();
createRoot(container).render(
  <Provider store={store}>
    <App />
  </Provider>,
);
