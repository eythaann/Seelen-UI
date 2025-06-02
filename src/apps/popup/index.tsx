import { getRootContainer } from '@shared/index';
import { StartThemingTool } from '@shared/ThemeLoader';
import { createRoot } from 'react-dom/client';

import { App } from './app';

import '../shared/styles/reset.css';
import '../shared/styles/colors.css';
import './global.css';

StartThemingTool();

const container = getRootContainer();
createRoot(container).render(<App />);
