import { emit } from '@tauri-apps/api/event';
import { useEffect } from 'react';
import { useSelector } from 'react-redux';

import { Item } from '../../item/infra';

import { Selectors } from '../../shared/store/app';

import { NetworkTM } from '../../../../shared/schemas/Placeholders';
import { WithWlanSelector } from './WlanSelector';

interface Props {
  module: NetworkTM;
}

function NetworkModuleItem({ module, ...rest }: Props) {
  const networkAdapters = useSelector(Selectors.networkAdapters);
  const defaultIp = useSelector(Selectors.networkLocalIp);
  const online = useSelector(Selectors.online);

  const usingAdapter = networkAdapters.find((i) => i.ipv4 === defaultIp) || null;

  return (
    <Item
      {...rest}
      extraVars={{
        online,
        interfaces: networkAdapters,
        usingInterface: usingAdapter,
      }}
      module={module}
    />
  );
}

export function NetworkModule({ module }: Props) {
  useEffect(() => {
    emit('register-network-events');
  }, []);

  return module.withWlanSelector ? (
    <WithWlanSelector>
      <NetworkModuleItem module={module} />
    </WithWlanSelector>
  ) : (
    <NetworkModuleItem module={module} />
  );
}
