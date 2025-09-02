import { startThemingTool } from '@seelen-ui/lib';
import { getRootContainer } from '@shared';
import { declareDocumentAsLayeredHitbox } from '@shared/layered';
import { removeDefaultWebviewActions } from '@shared/setup';
import { createRoot } from 'react-dom/client';

import { App } from './app';

import '@shared/styles/colors.css';
import '@shared/styles/reset.css';
import './styles/global.css';

removeDefaultWebviewActions();
await declareDocumentAsLayeredHitbox((e) => e.getAttribute('data-allow-mouse-events') === 'true');
await startThemingTool();

const container = getRootContainer();
createRoot(container).render(<App />);
