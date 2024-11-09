import { emit } from '@tauri-apps/api/event';
import React, { useEffect } from 'react';
import { useSelector } from 'react-redux';
import { invoke, MediaTM, SeelenCommand } from 'seelen-core';

import { Item } from '../../item/infra/infra';

import { Selectors } from '../../shared/store/app';

import { WithMediaControls } from './MediaControls';

interface Props {
  module: MediaTM;
}

function MediaModuleItem({ module, ...rest }: Props) {
  const {
    id,
    volume = 0,
    muted: isMuted = true,
  } = useSelector((state: any) =>
    Selectors.mediaOutputs(state).find((d) => d.is_default_multimedia),
  ) || {};

  const { volume: inputVolume = 0, muted: inputIsMuted = true } =
    useSelector((state: any) =>
      Selectors.mediaInputs(state).find((d) => d.is_default_multimedia),
    ) || {};

  const mediaSession =
    useSelector((state: any) => Selectors.mediaSessions(state).find((d) => d.default)) || null;

  function onWheel(e: React.WheelEvent) {
    const isUp = e.deltaY < 0;
    const level = Math.max(0, Math.min(1, volume + (isUp ? 0.02 : -0.02)));
    invoke(SeelenCommand.SetVolumeLevel, { id, level });
  }

  return (
    <Item
      {...rest}
      onWheel={module.withMediaControls ? onWheel : undefined}
      extraVars={{ volume, isMuted, inputVolume, inputIsMuted, mediaSession }}
      module={module}
    />
  );
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
