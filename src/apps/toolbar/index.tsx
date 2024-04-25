import { ErrorBoundary } from '../seelenweg/components/Error';
import { getRootContainer, setWindowAsFullSize } from '../utils';
import { wrapConsole } from '../utils/ConsoleWrapper';
import { registerDocumentEvents } from './events';
import { createRoot } from 'react-dom/client';

import { App } from './app';

import './styles/colors.css';
import './styles/variables.css';
import './styles/reset.css';
import './styles/global.css';

async function Main() {
  wrapConsole();
  const container = getRootContainer();

  setWindowAsFullSize();
  registerDocumentEvents();

  createRoot(container).render(
    <ErrorBoundary fallback={<div>Something went wrong</div>}>
      <App />
    </ErrorBoundary>,
  );
}

Main();
