import { NetworkTM, ToolbarModuleType } from '../../../../shared/schemas/Placeholders';
import { WithWlanSelector } from './WlanSelector';
import { emit } from '@tauri-apps/api/event';
import { useEffect } from 'react';
import { useSelector } from 'react-redux';

import { Item } from '../../item/infra';

import { Selectors } from '../../shared/store/app';

interface Props {
  module: NetworkTM;
}

function NetworkModuleItem({ module }: Props) {
  const networkAdapters = useSelector(Selectors.networkAdapters);
  const defaultIp = useSelector(Selectors.networkLocalIp);
  const online = useSelector(Selectors.online);

  const usingAdapter = networkAdapters.find((i) => i.ipv4 === defaultIp) || null;

  return (
    <Item
      extraVars={{
        online,
        interfaces: networkAdapters,
        usingInterface: usingAdapter,
      }}
      module={{
        ...module,
        onClick: module.withWlanSelector ? 'nothing' : module.onClick,
      }}
    />
  );
}

export function NetworkModule({ module }: Props) {
  useEffect(() => {
    if (!window.TOOLBAR_MODULES[ToolbarModuleType.Network]) {
      window.TOOLBAR_MODULES[ToolbarModuleType.Network] = true;
      emit('register-network-events');
    }
  }, []);

  return module.withWlanSelector ? (
    <WithWlanSelector>
      <NetworkModuleItem module={module} />
    </WithWlanSelector>
  ) : (
    <NetworkModuleItem module={module} />
  );
}
