import { ErrorBoundary } from '../seelenweg/components/Error';
import { wrapConsole } from '../shared/ConsoleWrapper';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useEffect } from 'react';
import { createRoot } from 'react-dom/client';
import { Provider } from 'react-redux';

import { Layout } from './modules/layout/infra';
import { loadStore, registerStoreEvents, store } from './modules/shared/store/infra';

import './styles/colors.css';
import './styles/variables.css';
import './styles/global.css';

async function Main() {
  wrapConsole();

  const container = document.getElementById('root');
  if (!container) {
    throw new Error('Root container not found');
  }

  await loadStore();
  await registerStoreEvents();

  const WrappedRoot = () => {
    useEffect(() => {
      getCurrentWebviewWindow().show();
      let view = getCurrentWebviewWindow();
      view.emitTo(view.label, 'complete-setup');
      view.emit('register-colors-events');
    }, []);

    return (
      <Provider store={store}>
        <ErrorBoundary fallback={<div>Something went wrong</div>}>
          <Layout />
        </ErrorBoundary>
      </Provider>
    );
  };

  createRoot(container).render(<WrappedRoot />);
}

Main();
