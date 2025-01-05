import { UserToolbarItem } from '@seelen-ui/lib/types';

import { Item } from '../../item/infra/infra';

import { WithUserHome } from './UserHome';

interface Props {
  module: UserToolbarItem;
}

function UserModuleItem({ module, ...rest }: Props) {
  return (
    <Item
      {...rest}
      module={module}
    />
  );
}

export function UserModule({ module }: Props) {
  return module.withUserFolder ? (
    <WithUserHome>
      <UserModuleItem module={module} />
    </WithUserHome>
  ) : (
    <UserModuleItem module={module} />
  );
}
