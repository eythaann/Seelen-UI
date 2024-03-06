import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../../../components/SettingsBox';
import { InputNumber, Select } from 'antd';
import { useDispatch } from 'react-redux';

import { useAppSelector } from '../../../shared/app/hooks';
import { Rect } from '../../../shared/app/Rect';
import { GeneralSettingsSelectors } from '../../../shared/app/selectors';
import { OptionsFromEnum } from '../../../shared/app/utils';
import { GeneralSettingsActions } from '../app';

import { CrossMonitorMoveBehaviour, UnmanagedWindowOperationBehaviour } from '../domain';

export const OthersConfigs = () => {
  const resizeDelta = useAppSelector(GeneralSettingsSelectors.resizeDelta);
  const unmanagedWindowOpBehaviour = useAppSelector(GeneralSettingsSelectors.unmanagedWindowOperationBehaviour);
  const crossMonitorMoveBehaviour = useAppSelector(GeneralSettingsSelectors.crossMonitorMoveBehaviour);
  const invisibleBorders = useAppSelector(GeneralSettingsSelectors.invisibleBorders);

  const dispatch = useDispatch();

  const onChangeInvisibleBorders = (side: keyof Rect, value: number | null) => {
    dispatch(
      GeneralSettingsActions.setInvisibleBorders({
        ...invisibleBorders,
        [side]: value || 0,
      }),
    );
  };

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
        <SettingsSubGroup label="Invisible borders">
          <SettingsOption>
            <span>Left</span>
            <InputNumber value={invisibleBorders.left} onChange={onChangeInvisibleBorders.bind(this, 'left')} />
          </SettingsOption>
          <SettingsOption>
            <span>Top</span>
            <InputNumber value={invisibleBorders.top} onChange={onChangeInvisibleBorders.bind(this, 'top')} />
          </SettingsOption>
          <SettingsOption>
            <span>Right</span>
            <InputNumber value={invisibleBorders.right} onChange={onChangeInvisibleBorders.bind(this, 'right')} />
          </SettingsOption>
          <SettingsOption>
            <span>Bottom</span>
            <InputNumber value={invisibleBorders.bottom} onChange={onChangeInvisibleBorders.bind(this, 'bottom')} />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

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
