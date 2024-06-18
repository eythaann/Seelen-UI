import { NetworkTM, ToolbarModuleType } from '../../../shared/schemas/Placeholders';
import { emit } from '@tauri-apps/api/event';
import { useCallback, useEffect, useState } from 'react';
import { useSelector } from 'react-redux';

import { Item } from '../item/infra';

import { Selectors } from '../shared/store/app';

interface Props {
  module: NetworkTM;
}

export function NetworkModule({ module }: Props) {
  const networkAdapters = useSelector(Selectors.networkAdapters);
  const defaultIp = useSelector(Selectors.networkLocalIp);
  const online = useSelector(Selectors.online);

  useEffect(() => {
    if (!window.TOOLBAR_MODULES[ToolbarModuleType.Network]) {
      window.TOOLBAR_MODULES[ToolbarModuleType.Network] = true;
      emit('register-network-events');
    }
  }, []);

  const usingAdapter = networkAdapters.find((i) => i.ipv4 === defaultIp) || null;

  return (
    <Item
      extraVars={{
        online,
        interfaces: networkAdapters,
        usingInterface: usingAdapter,
      }}
      module={module}
    />
  );
}
