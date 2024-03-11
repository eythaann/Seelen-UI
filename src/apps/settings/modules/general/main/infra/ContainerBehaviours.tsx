import { SettingsGroup, SettingsOption } from '../../../../components/SettingsBox';
import { Select, Switch } from 'antd';
import { useDispatch } from 'react-redux';

import { useAppSelector } from '../../../shared/app/hooks';
import { GeneralSettingsSelectors } from '../../../shared/app/selectors';
import { OptionsFromEnum } from '../../../shared/app/utils';
import { GeneralSettingsActions } from '../app';

import { WindowContainerBehaviour, WindowHidingBehaviour } from '../domain';

export const ContainerBehaviors = () => {
  const autoStackinByCategory = useAppSelector(GeneralSettingsSelectors.autoStackinByCategory);
  const windowContainerBehaviour = useAppSelector(GeneralSettingsSelectors.windowContainerBehaviour);
  const windowHidingBehaviour = useAppSelector(GeneralSettingsSelectors.windowHidingBehaviour);

  const dispatch = useDispatch();

  const onChangeAutoStackinByCategory = (value: boolean) => {
    dispatch(GeneralSettingsActions.setAutoStackinByCategory(value));
  };

  const onChangeHiddingBehaviour = (value: WindowHidingBehaviour) => {
    dispatch(GeneralSettingsActions.setWindowHidingBehaviour(value));
  };

  const onChangeContainerBehaviour = (value: WindowContainerBehaviour) => {
    dispatch(GeneralSettingsActions.setWindowContainerBehaviour(value));
  };

  return (
    <SettingsGroup>
      <div>
        <SettingsOption>
          <span>Window container behaviour</span>
          <Select
            value={windowContainerBehaviour}
            options={OptionsFromEnum(WindowContainerBehaviour)}
            onChange={onChangeContainerBehaviour}
          />
        </SettingsOption>
        <SettingsOption>
          <span>Auto Stack by category (append if same category)</span>
          <Switch value={autoStackinByCategory} onChange={onChangeAutoStackinByCategory} />
        </SettingsOption>
      </div>
      <SettingsOption>
        <span>Window hiding behaviour</span>
        <Select
          value={windowHidingBehaviour}
          options={OptionsFromEnum(WindowHidingBehaviour)}
          onChange={onChangeHiddingBehaviour}
        />
      </SettingsOption>
    </SettingsGroup>
  );
};
