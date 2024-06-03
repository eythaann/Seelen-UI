import { PowerToolbarModule, ToolbarModuleType } from '../../../utils/schemas/Placeholders';
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

  useEffect(() => {
    if (!window.TOOLBAR_MODULES[ToolbarModuleType.Power]) {
      window.TOOLBAR_MODULES[ToolbarModuleType.Power] = true;
      emit('register-power-events');
    }
  }, []);

  return <Item extraVars={{ power }} module={module} />;
}