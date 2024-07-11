import { MediaTM, ToolbarModuleType } from '../../../../shared/schemas/Placeholders';
import { WithMediaControls } from './MediaControls';
import { emit } from '@tauri-apps/api/event';
import { useEffect } from 'react';
import { useSelector } from 'react-redux';

import { Item } from '../../item/infra';

import { Selectors } from '../../shared/store/app';

interface Props {
  module: MediaTM;
}

function MediaModuleItem({ module }: Props) {
  const volume = useSelector(Selectors.mediaVolume);
  const isMuted = useSelector(Selectors.mediaMuted);

  return (
    <Item
      extraVars={{ volume, isMuted }}
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
