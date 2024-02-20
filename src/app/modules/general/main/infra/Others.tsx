import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../../../components/SettingsBox';
import { InputNumber, Select } from 'antd';

import { useAppSelector } from '../../../shared/app/hooks';
import { GeneralSettingsSelectors } from '../../../shared/app/selectors';
import { OptionsFromEnum } from '../../../shared/app/utils';

import { CrossMonitorMoveBehaviour, UnmanagedWindowOperationBehaviour } from '../domain';

export const OthersConfigs = () => {
  const resizeDelta = useAppSelector(GeneralSettingsSelectors.resizeDelta);
  const unmanagedWindowOpBehaviour = useAppSelector(GeneralSettingsSelectors.unmanagedWindowOperationBehaviour);
  const crossMonitorMoveBehaviour = useAppSelector(GeneralSettingsSelectors.crossMonitorMoveBehaviour);
  const invisibleBorders = useAppSelector(GeneralSettingsSelectors.invisibleBorders);

  return (
    <>
      <SettingsGroup>
        <SettingsSubGroup label="Invisible borders">
          <SettingsOption>
            <span>Left</span>
            <InputNumber value={invisibleBorders.left} />
          </SettingsOption>
          <SettingsOption>
            <span>Top</span>
            <InputNumber value={invisibleBorders.top} />
          </SettingsOption>
          <SettingsOption>
            <span>Right</span>
            <InputNumber value={invisibleBorders.right} />
          </SettingsOption>
          <SettingsOption>
            <span>Bottom</span>
            <InputNumber value={invisibleBorders.bottom} />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>
      <SettingsGroup>
        <SettingsOption>
          <span>Resize delta</span>
          <InputNumber value={resizeDelta} />
        </SettingsOption>
        <SettingsOption>
          <span>Cross monitor move behaviour - Swap Insert</span>
          <Select value={crossMonitorMoveBehaviour} options={OptionsFromEnum(CrossMonitorMoveBehaviour)} />
        </SettingsOption>
        <SettingsOption>
          <span>Unmanaged window operation behaviour</span>
          <Select value={unmanagedWindowOpBehaviour} options={OptionsFromEnum(UnmanagedWindowOperationBehaviour)} />
        </SettingsOption>
      </SettingsGroup>
    </>
  );
};
