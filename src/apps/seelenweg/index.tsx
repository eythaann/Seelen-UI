
import { wrapConsole } from '../ConsoleWrapper';
import { getCurrent } from '@tauri-apps/api/window';
import { ConfigProvider, theme } from 'antd';
import { useEffect } from 'react';
import { createRoot } from 'react-dom/client';
import { Provider } from 'react-redux';

import { store } from './modules/shared/store/infra';

import './styles/colors.css';
import './styles/reset.css';
import './styles/global.css';

wrapConsole();

const RouletteNode = document.getElementById('root');

const root = createRoot(RouletteNode || document.body);

const WrappedRoot = () => {
  useEffect(() => {
    getCurrent().show();
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

    </ConfigProvider>
  </Provider>;
};

root.render(<WrappedRoot/>);