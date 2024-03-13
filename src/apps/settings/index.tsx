import { wrapConsole } from '../ConsoleWrapper';
import { getCurrent } from '@tauri-apps/api/webviewWindow';
import { ConfigProvider, theme } from 'antd';
import { useEffect } from 'react';
import { createRoot } from 'react-dom/client';
import { Provider } from 'react-redux';

import { LoadSettingsToStore, store } from './modules/shared/infrastructure/store';

import { App } from './app';

import './styles/colors.css';
import './styles/variables.css';
import './styles/reset.css';
import './styles/global.css';

(async function main() {
  wrapConsole();
  await LoadSettingsToStore();

  const container = document.getElementById('root');
  if (container) {
    const WrappedRoot = () => {
      useEffect(() => {
        setTimeout(() => {
          getCurrent().show();
        }, 0);
      });

      return (
        <Provider store={store}>
          <ConfigProvider
            componentSize="small"
            theme={{
              algorithm: window.matchMedia('(prefers-color-scheme: dark)').matches
                ? theme.darkAlgorithm
                : theme.defaultAlgorithm,
            }}
          >
            <App />
          </ConfigProvider>
        </Provider>
      );
    };

    createRoot(container).render(<WrappedRoot />);
  }
})();
