import { ErrorBoundary } from '../seelenweg/components/Error';
import { toPhysicalPixels } from '../utils';
import { ErrorFallback } from './components/Error';
import { emitTo } from '@tauri-apps/api/event';
import { getCurrent } from '@tauri-apps/api/webviewWindow';
import { useEffect } from 'react';
import { useSelector } from 'react-redux';

import { ToolBar } from './modules/main/infra';

import { Selectors } from './modules/shared/store/app';

import { HITBOX_TARGET, SELF_TARGET } from './modules/shared/utils/domain';

export function App() {
  const structure = useSelector(Selectors.placeholder);
  const height = useSelector(Selectors.settings.height);

  useEffect(() => {
    emitTo(HITBOX_TARGET, 'init').then(async () => {
      await getCurrent().show();
      await emitTo(SELF_TARGET, 'complete-setup', toPhysicalPixels(height));
    });
  }, []);

  if (!structure) {
    return <ErrorFallback msg="NO PLACEHOLDER FOUND. PLEASE TRY REINSTALL THE APP" />;
  }

  return (
    <ErrorBoundary fallback={<ErrorFallback />}>
      <ToolBar structure={structure} />
    </ErrorBoundary>
  );
}
