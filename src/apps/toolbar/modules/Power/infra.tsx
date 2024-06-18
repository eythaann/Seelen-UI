import { PowerToolbarModule, ToolbarModuleType } from '../../../shared/schemas/Placeholders';
import { emit } from '@tauri-apps/api/event';
import { useEffect } from 'react';
import { useSelector } from 'react-redux';

import { Item } from '../item/infra';

import { Selectors } from '../shared/store/app';

interface Props {
  module: PowerToolbarModule;
}

export function PowerModule({ module }: Props) {
  const power = useSelector(Selectors.powerStatus);
  const batteries = useSelector(Selectors.batteries);

  useEffect(() => {
    if (!window.TOOLBAR_MODULES[ToolbarModuleType.Power]) {
      window.TOOLBAR_MODULES[ToolbarModuleType.Power] = true;
      emit('register-power-events');
    }
  }, []);

  if (!batteries.length) {
    return null;
  }

  return (
    <Item
      extraVars={{
        power,
        batteries,
        battery: batteries[0],
      }}
      module={module}
    />
  );
}
