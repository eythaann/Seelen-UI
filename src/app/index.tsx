import { appWindow } from '@tauri-apps/api/window';
import { ConfigProvider, theme } from 'antd';
import { useEffect } from 'react';
import { createRoot } from 'react-dom/client';
import { Provider } from 'react-redux';

import { Roulette } from './modules/roulette/infra';
import { LoadSettingsToStore, store } from './modules/shared/infrastructure/store';

import { App } from './app';

import './styles/colors.css';
import './styles/variables.css';
import './styles/reset.css';
import './styles/global.css';

LoadSettingsToStore();

const SettingsNode = document.getElementById('root');
const RouletteNode = document.getElementById('root-roulette');

const root = createRoot(SettingsNode || RouletteNode || document.body);
const Root = RouletteNode ? Roulette : App;

const WrappedRoot = () => {
  async function setupAppWindow() {
    appWindow.show();
  }

  useEffect(() => {
    setupAppWindow();
  }, []);

  return <Provider store={store}>
    <ConfigProvider
      componentSize="small"
      theme={{
        algorithm: window.matchMedia('(prefers-color-scheme: dark)').matches
          ? theme.darkAlgorithm
          : theme.defaultAlgorithm,
      }}
    >
      <Root/>
    </ConfigProvider>
  </Provider>;
};

root.render(<WrappedRoot/>);
