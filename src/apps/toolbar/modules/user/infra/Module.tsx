import { UserToolbarItem } from '@seelen-ui/lib/types';
import { useState } from 'react';

import { Item } from '../../item/infra/infra';

import { WithUserHome } from './UserHome';

interface Props {
  module: UserToolbarItem;
  active?: boolean;
}

function UserModuleItem({ module, active, ...rest }: Props) {
  return (
    <Item
      {...rest}
      active={active}
      module={module}
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
