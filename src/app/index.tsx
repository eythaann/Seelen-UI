import { createRoot } from 'react-dom/client';
import { Provider } from 'react-redux';

import { store } from './modules/shared/infrastructure/store';

import { App } from './app';

import './styles/colors.css';
import './styles/variables.css';
import './styles/reset.css';
import './styles/global.css';

const domNode = document.getElementById('root');
if (domNode) {
  const root = createRoot(domNode);
  root.render(<Provider store={store}>
    <App/>
  </Provider>);
}