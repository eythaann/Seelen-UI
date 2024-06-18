import { wrapConsole } from '../shared/ConsoleWrapper';
import { createRoot } from 'react-dom/client';

import { App } from './app';

import './styles/colors.css';
import './styles/reset.css';
import './styles/global.css';

async function main() {
  wrapConsole();
  const root = createRoot(document.getElementById('root') || document.body);
  root.render(<App/>);
}

main();