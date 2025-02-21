import { PowerToolbarItem } from '@seelen-ui/lib/types';
import { useSelector } from 'react-redux';

import { Item } from '../item/infra/infra';

import { Selectors } from '../shared/store/app';

interface Props {
  module: PowerToolbarItem;
}

export function PowerModule({ module }: Props) {
  const power = useSelector(Selectors.powerStatus);
  const powerPlan = useSelector(Selectors.powerPlan);
  const batteries = useSelector(Selectors.batteries);

  if (!batteries.length) {
    return null;
  }

  return (
    <Item
      extraVars={{
        power,
        powerPlan,
        batteries,
        battery: batteries[0],
      }}
      module={module}
    />
  );
}
