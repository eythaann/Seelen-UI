import { MediaTM } from '../../../../shared/schemas/Placeholders';
import { WithMediaControls } from './MediaControls';
import { emit } from '@tauri-apps/api/event';
import { useEffect } from 'react';
import { useSelector } from 'react-redux';

import { Item } from '../../item/infra';

import { Selectors } from '../../shared/store/app';

interface Props {
  module: MediaTM;
}

function MediaModuleItem({ module, ...rest }: Props) {
  const { volume = 0, muted: isMuted = false } =
    useSelector((state: any) => Selectors.mediaOutputs(state).find((d) => d.is_default_multimedia)) || {};

  return <Item {...rest} extraVars={{ volume, isMuted }} module={module} />;
}

export function MediaModule({ module }: Props) {
  useEffect(() => {
    emit('register-media-events');
  }, []);

  return module.withMediaControls ? (
    <WithMediaControls>
      <MediaModuleItem module={module} />
    </WithMediaControls>
  ) : (
    <MediaModuleItem module={module} />
  );
}
