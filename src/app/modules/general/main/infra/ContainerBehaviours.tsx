import { SettingsGroup, SettingsOption } from '../../../../components/SettingsBox';
import { Select, Switch } from 'antd';

import { useAppSelector } from '../../../shared/app/hooks';
import { GeneralSettingsSelectors } from '../../../shared/app/selectors';
import { OptionsFromEnum } from '../../../shared/app/utils';

import { WindowContainerBehaviour, WindowHidingBehaviour } from '../domain';

export const ContainerBehaviors = () => {
  const autoStackinByCategory = useAppSelector(GeneralSettingsSelectors.autoStackinByCategory);
  const windowContainerBehaviour = useAppSelector(GeneralSettingsSelectors.windowContainerBehaviour);
  const windowHidingBehaviour = useAppSelector(GeneralSettingsSelectors.windowHidingBehaviour);

  return <SettingsGroup>
    <div>
      <SettingsOption>
        <span>Window container behaviour</span>
        <Select
          value={windowContainerBehaviour}
          options={OptionsFromEnum(WindowContainerBehaviour)}
        />
      </SettingsOption>
      <SettingsOption>
        <span>Auto Stack by category (append if same category)</span>
        <Switch value={autoStackinByCategory} />
      </SettingsOption>
    </div>
    <SettingsOption>
      <span>Window hiding behaviour</span>
      <Select
        value={windowHidingBehaviour}
        options={OptionsFromEnum(WindowHidingBehaviour)}
      />
    </SettingsOption>
  </SettingsGroup>;
};