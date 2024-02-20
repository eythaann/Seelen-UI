import { ConfigProvider } from 'antd';
import { createRoot } from 'react-dom/client';
import { Provider } from 'react-redux';

import { LoadSettingsToStore, store } from './modules/shared/infrastructure/store';

import { App } from './app';

import './styles/colors.css';
import './styles/variables.css';
import './styles/reset.css';
import './styles/global.css';

LoadSettingsToStore();

const domNode = document.getElementById('root');
if (domNode) {
  const root = createRoot(domNode);
  root.render(
    <Provider store={store}>
      <ConfigProvider componentSize="small">
        <App />
      </ConfigProvider>
    </Provider>,
  );
}
