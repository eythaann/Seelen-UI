import { MediaTM, ToolbarModuleType } from '../../../../shared/schemas/Placeholders';
import { WithMediaControls } from './MediaControls';
import { emit } from '@tauri-apps/api/event';
import { useEffect } from 'react';

import { Item } from '../../item/infra';

interface Props {
  module: MediaTM;
}

function MediaModuleItem({ module }: Props) {
  return (
    <Item
      extraVars={{}}
      module={{
        ...module,
        onClick: module.withMediaControls ? 'nothing' : module.onClick,
      }}
    />
  );
}

export function MediaModule({ module }: Props) {
  useEffect(() => {
    if (!window.TOOLBAR_MODULES[ToolbarModuleType.Media]) {
      window.TOOLBAR_MODULES[ToolbarModuleType.Media] = true;
      emit('register-media-events');
    }
  }, []);

  return module.withMediaControls ? (
    <WithMediaControls>
      <MediaModuleItem module={module} />
    </WithMediaControls>
  ) : (
    <MediaModuleItem module={module} />
  );
}
