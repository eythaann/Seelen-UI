import { SettingsGroup, SettingsOption } from '../../../components/SettingsBox';
import { Select } from 'antd';
import { useDispatch } from 'react-redux';

import { useAppSelector } from '../../shared/app/hooks';
import { ContainerTopBarSelectors } from '../../shared/app/selectors';
import { OptionsFromEnum } from '../../shared/app/utils';
import { ContainerTopBarActions } from './app';

import { ContainerTopBarMode } from './domain';

export const ContainerTopBarSettings = () => {
  const mode = useAppSelector(ContainerTopBarSelectors.mode);

  const dispatch = useDispatch();

  const updateTopBarMode = (value: ContainerTopBarMode) => {
    dispatch(ContainerTopBarActions.setMode(value));
  };

  return (
    <SettingsGroup>
      <SettingsOption>
        <span>Container Top Bar - Tabs</span>
        <Select value={mode} options={OptionsFromEnum(ContainerTopBarMode)} onChange={updateTopBarMode} />
      </SettingsOption>
    </SettingsGroup>
  );
};
