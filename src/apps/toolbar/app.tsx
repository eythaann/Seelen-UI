import { ErrorBoundary } from '../seelenweg/components/Error';
import { ErrorFallback } from './components/Error';
import { emitTo } from '@tauri-apps/api/event';
import { getCurrent } from '@tauri-apps/api/webviewWindow';
import { useEffect } from 'react';
import { useSelector } from 'react-redux';

import { ToolBar } from './modules/main/infra';

import { Selectors } from './modules/shared/store/app';

async function onMount() {
  let view = getCurrent();
  await emitTo(view.label.replace('/', '-hitbox/'), 'init');
  await view.show();
}

export function App() {
  const structure = useSelector(Selectors.placeholder);

  useEffect(() => {
    onMount();
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
