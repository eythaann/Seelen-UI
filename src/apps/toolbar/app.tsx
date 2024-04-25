import { updateHitbox } from './events';
import { emitTo } from '@tauri-apps/api/event';
import { getCurrent } from '@tauri-apps/api/webviewWindow';
import { useEffect } from 'react';

import { ToolBar } from './modules/main/infra';

import { HITBOX_TARGET } from './modules/shared/utils/domain';

export function App() {
  useEffect(() => {
    emitTo(HITBOX_TARGET, 'init').then(() => {
      updateHitbox();
      getCurrent().show();
    });
  }, []);

  return <ToolBar/>;
}