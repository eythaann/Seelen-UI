
import { getRootContainer } from '../shared';
import { wrapConsole } from '../shared/ConsoleWrapper';
import { App } from './App';
import { createRoot } from 'react-dom/client';

import './styles/reset.css';
import './styles/global.css';
import './styles/colors.css';

async function Main() {
  wrapConsole();
  const container = getRootContainer();
  createRoot(container).render(<App />);
}

Main();