import { getRootContainer } from '../shared';
import { wrapConsole } from '../shared/ConsoleWrapper';
import { useDarkMode } from '../shared/styles';
import i18n, { loadTranslations } from './i18n';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { ConfigProvider, theme } from 'antd';
import { useEffect } from 'react';
import { createRoot } from 'react-dom/client';
import { I18nextProvider } from 'react-i18next';
import { Provider } from 'react-redux';

import { LoadSettingsToStore, store } from './modules/shared/store/infra';

import { App } from './app';

import './styles/colors.css';
import './styles/variables.css';
import './styles/reset.css';
import './styles/global.css';

(async function main() {
  getCurrentWebviewWindow().show();
  wrapConsole();
  const container = getRootContainer();

  await LoadSettingsToStore();
  await loadTranslations();

  const WrappedRoot = () => {
    useEffect(() => {
      let splashscreen = document.getElementById('splashscreen');
      splashscreen?.classList.add('vanish');
      setTimeout(() => splashscreen?.classList.add('hidden'), 300);
    }, []);

    const isDarkMode = useDarkMode();

    return (
      <Provider store={store}>
        <I18nextProvider i18n={i18n}>
          <ConfigProvider
            componentSize="small"
            theme={{
              algorithm: isDarkMode ? theme.darkAlgorithm : theme.defaultAlgorithm,
            }}
          >
            <App />
          </ConfigProvider>
        </I18nextProvider>
      </Provider>
    );
  };

  createRoot(container).render(<WrappedRoot />);
})();
