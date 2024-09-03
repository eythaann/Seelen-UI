import { getRootContainer } from '../shared';
import { wrapConsole } from '../shared/ConsoleWrapper';
import { App } from './App';
import { registerDocumentEvents } from './events';
import { initStore, store } from './store';
import { createRoot } from 'react-dom/client';
import { Provider } from 'react-redux';

import './styles/reset.css';
import './styles/colors.css';

async function Main() {
  wrapConsole();
  await initStore();
  registerDocumentEvents();

  createRoot(getRootContainer()).render(
    <Provider store={store}>
      <App />
    </Provider>,
  );
}

Main();
