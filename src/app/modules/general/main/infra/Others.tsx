import { SettingsGroup, SettingsOption } from '../../../../components/SettingsBox';
import { InputNumber, Select } from 'antd';
import { useDispatch } from 'react-redux';

import { useAppSelector } from '../../../shared/app/hooks';
import { GeneralSettingsSelectors } from '../../../shared/app/selectors';
import { OptionsFromEnum } from '../../../shared/app/utils';
import { GeneralSettingsActions } from '../app';

import { CrossMonitorMoveBehaviour, UnmanagedWindowOperationBehaviour } from '../domain';

export const OthersConfigs = () => {
  const resizeDelta = useAppSelector(GeneralSettingsSelectors.resizeDelta);
  const unmanagedWindowOpBehaviour = useAppSelector(GeneralSettingsSelectors.unmanagedWindowOperationBehaviour);
  const crossMonitorMoveBehaviour = useAppSelector(GeneralSettingsSelectors.crossMonitorMoveBehaviour);

  const dispatch = useDispatch();

  const onChangeResizeDelta = (value: number | null) => {
    dispatch(GeneralSettingsActions.setResizeDelta(value || 0));
  };

  const onChangeMonitorBehaviour = (value: CrossMonitorMoveBehaviour) => {
    dispatch(GeneralSettingsActions.setCrossMonitorMoveBehaviour(value));
  };

  const onChangeUnmanageBehaviour = (value: UnmanagedWindowOperationBehaviour) => {
    dispatch(GeneralSettingsActions.setUnmanagedWindowOperationBehaviour(value));
  };

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <span>Resize delta</span>
          <InputNumber value={resizeDelta} onChange={onChangeResizeDelta} />
        </SettingsOption>
        <SettingsOption>
          <span>Cross monitor move behaviour</span>
          <Select
            value={crossMonitorMoveBehaviour}
            options={OptionsFromEnum(CrossMonitorMoveBehaviour)}
            onChange={onChangeMonitorBehaviour}
          />
        </SettingsOption>
        <SettingsOption>
          <span>Unmanaged window operation behaviour</span>
          <Select
            value={unmanagedWindowOpBehaviour}
            options={OptionsFromEnum(UnmanagedWindowOperationBehaviour)}
            onChange={onChangeUnmanageBehaviour}
          />
        </SettingsOption>
      </SettingsGroup>
    </>
  );
};
