import { ErrorBoundary } from '../seelenweg/components/Error';
import { toPhysicalPixels } from '../utils';
import { ErrorFallback } from './components/Error';
import { emitTo } from '@tauri-apps/api/event';
import { getCurrent } from '@tauri-apps/api/webviewWindow';
import { useEffect } from 'react';

import { ToolBar } from './modules/main/infra';

import { HITBOX_TARGET, SELF_TARGET } from './modules/shared/utils/domain';

export function App() {
  const height = 30;

  useEffect(() => {
    emitTo(HITBOX_TARGET, 'init').then(async () => {
      await getCurrent().show();
      await emitTo(SELF_TARGET, 'complete-setup', toPhysicalPixels(height));
    });
  }, []);

  return <ErrorBoundary fallback={<ErrorFallback />}><ToolBar/></ErrorBoundary>;
}