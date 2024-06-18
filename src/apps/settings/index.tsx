import { getRootContainer } from '../shared';
import { wrapConsole } from '../shared/ConsoleWrapper';
import { getCurrent } from '@tauri-apps/api/webviewWindow';
import { ConfigProvider, theme } from 'antd';
import { useEffect } from 'react';
import { createRoot } from 'react-dom/client';
import { Provider } from 'react-redux';

import { LoadSettingsToStore, store } from './modules/shared/store/infra';

import { App } from './app';

import './styles/colors.css';
import './styles/variables.css';
import './styles/reset.css';
import './styles/global.css';

(async function main() {
  getCurrent().show();
  wrapConsole();
  const container = getRootContainer();

  await LoadSettingsToStore();

  const WrappedRoot = () => {
    useEffect(() => {
      let splashscreen = document.getElementById('splashscreen');
      splashscreen?.classList.add('vanish');
      setTimeout(() => splashscreen?.classList.add('hidden'), 300);
    }, []);

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
})();
