import { NetworkToolbarItem } from '@seelen-ui/lib/types';
import { emit } from '@tauri-apps/api/event';
import { useEffect, useState } from 'react';
import { useSelector } from 'react-redux';

import { Item } from '../../item/infra/infra';

import { Selectors } from '../../shared/store/app';

import { WithWlanSelector } from './WlanSelector';

interface Props {
  active?: boolean;
  module: NetworkToolbarItem;
}

function NetworkModuleItem({ module, active, ...rest }: Props) {
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
      active={active}
    />
  );
}

export function NetworkModule({ module }: Props) {
  const [ open, setOpen ] = useState(false);

  useEffect(() => {
    emit('register-network-events');
  }, []);

  return module.withWlanSelector ? (
    <WithWlanSelector setActive={setOpen}>
      <NetworkModuleItem module={module} active={open} />
    </WithWlanSelector>
  ) : (
    <NetworkModuleItem module={module} />
  );
}
