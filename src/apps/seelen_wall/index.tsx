import { startThemingTool } from '@seelen-ui/lib';
import { removeDefaultWebviewActions } from '@shared/setup';
import { createRoot } from 'react-dom/client';

import { App } from './app';

import { getRootContainer } from '../shared';

import '../shared/styles/colors.css';
import '../shared/styles/reset.css';
import './styles/global.css';

removeDefaultWebviewActions();
await startThemingTool();

const container = getRootContainer();
createRoot(container).render(<App />);
