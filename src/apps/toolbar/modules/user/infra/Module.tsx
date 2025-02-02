import { UserToolbarItem } from '@seelen-ui/lib/types';
import { useState } from 'react';
import { useSelector } from 'react-redux';

import { Item } from '../../item/infra/infra';

import { Selectors } from '../../shared/store/app';

import { WithUserHome } from './UserHome';

interface Props {
  module: UserToolbarItem;
  active?: boolean;
}

function UserModuleItem({ module, active, ...rest }: Props) {
  const user = useSelector(Selectors.user);
  return (
    <Item
      {...rest}
      active={active}
      module={module}
      extraVars={{ user }}
    />
  );
}

export function UserModule({ module }: Props) {
  const [isActive, setIsActive] = useState(false);

  return module.withUserFolder ? (
    <WithUserHome setOpen={setIsActive}>
      <UserModuleItem module={module} active={isActive} />
    </WithUserHome>
  ) : (
    <UserModuleItem module={module} />
  );
}
