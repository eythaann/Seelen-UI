import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../../../components/SettingsBox';
import { InputNumber } from 'antd';

import { useAppSelector } from '../../../shared/app/hooks';
import { GeneralSettingsSelectors } from '../../../shared/app/selectors';

export const GlobalPaddings = () => {
  const containerPadding = useAppSelector(GeneralSettingsSelectors.containerPadding);
  const workspacePadding = useAppSelector(GeneralSettingsSelectors.workspacePadding);
  const workAreaOffset = useAppSelector(GeneralSettingsSelectors.globalWorkAreaOffset);

  return <SettingsGroup>
    <div>
      <SettingsOption>
        <span>Default gap between containers</span>
        <InputNumber value={containerPadding} />
      </SettingsOption>
      <SettingsOption>
        <span>Default workspaces padding</span>
        <InputNumber value={workspacePadding} />
      </SettingsOption>
    </div>
    <SettingsSubGroup label="Default monitors offset (margins)">
      <SettingsOption>
        <span>Left</span>
        <InputNumber value={workAreaOffset.left} />
      </SettingsOption>
      <SettingsOption>
        <span>Top</span>
        <InputNumber value={workAreaOffset.top} />
      </SettingsOption>
      <SettingsOption>
        <span>Right</span>
        <InputNumber value={workAreaOffset.right} />
      </SettingsOption>
      <SettingsOption>
        <span>Bottom</span>
        <InputNumber value={workAreaOffset.bottom} />
      </SettingsOption>
    </SettingsSubGroup>
  </SettingsGroup>;
};