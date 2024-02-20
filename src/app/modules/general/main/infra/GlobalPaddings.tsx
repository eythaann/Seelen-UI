import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../../../components/SettingsBox';
import { InputNumber } from 'antd';
import { useDispatch } from 'react-redux';

import { useAppSelector } from '../../../shared/app/hooks';
import { Rect } from '../../../shared/app/Rect';
import { GeneralSettingsSelectors } from '../../../shared/app/selectors';
import { GeneralSettingsActions } from '../app';

export const GlobalPaddings = () => {
  const containerPadding = useAppSelector(GeneralSettingsSelectors.containerPadding);
  const workspacePadding = useAppSelector(GeneralSettingsSelectors.workspacePadding);
  const workAreaOffset = useAppSelector(GeneralSettingsSelectors.globalWorkAreaOffset);

  const dispatch = useDispatch();

  const onChangeGlobalOffset = (side: keyof Rect.plain, value: number | null) => {
    dispatch(
      GeneralSettingsActions.setGlobalWorkAreaOffset({
        ...workAreaOffset,
        [side]: value || 0,
      }),
    );
  };

  const onChangeDefaultGap = (value: number | null) => {
    dispatch(GeneralSettingsActions.setContainerPadding(value || 0));
  };

  const onChangeDefaultPadding = (value: number | null) => {
    dispatch(GeneralSettingsActions.setWorkspacePadding(value || 0));
  };

  return (
    <SettingsGroup>
      <div>
        <SettingsOption>
          <span>Default gap between containers</span>
          <InputNumber value={containerPadding} onChange={onChangeDefaultGap} />
        </SettingsOption>
        <SettingsOption>
          <span>Default workspaces padding</span>
          <InputNumber value={workspacePadding} onChange={onChangeDefaultPadding} />
        </SettingsOption>
      </div>
      <SettingsSubGroup label="Default monitors offset (margins)">
        <SettingsOption>
          <span>Left</span>
          <InputNumber value={workAreaOffset.left} onChange={onChangeGlobalOffset.bind(this, 'left')} />
        </SettingsOption>
        <SettingsOption>
          <span>Top</span>
          <InputNumber value={workAreaOffset.top} onChange={onChangeGlobalOffset.bind(this, 'top')} />
        </SettingsOption>
        <SettingsOption>
          <span>Right</span>
          <InputNumber value={workAreaOffset.right} onChange={onChangeGlobalOffset.bind(this, 'right')} />
        </SettingsOption>
        <SettingsOption>
          <span>Bottom</span>
          <InputNumber value={workAreaOffset.bottom} onChange={onChangeGlobalOffset.bind(this, 'bottom')} />
        </SettingsOption>
      </SettingsSubGroup>
    </SettingsGroup>
  );
};
