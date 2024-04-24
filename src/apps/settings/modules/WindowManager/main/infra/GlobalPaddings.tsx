import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../../../components/SettingsBox';
import { InputNumber } from 'antd';
import { useDispatch } from 'react-redux';

import { useAppSelector } from '../../../shared/app/hooks';
import { Rect } from '../../../shared/app/Rect';
import { SeelenWmSelectors } from '../../../shared/app/selectors';
import { WManagerSettingsActions } from '../app';

export const GlobalPaddings = () => {
  const containerPadding = useAppSelector(SeelenWmSelectors.containerPadding);
  const workspacePadding = useAppSelector(SeelenWmSelectors.workspacePadding);
  const workAreaOffset = useAppSelector(SeelenWmSelectors.globalWorkAreaOffset);

  const dispatch = useDispatch();

  const onChangeGlobalOffset = (side: keyof Rect, value: number | null) => {
    dispatch(
      WManagerSettingsActions.setGlobalWorkAreaOffset({
        ...workAreaOffset,
        [side]: value || 0,
      }),
    );
  };

  const onChangeDefaultGap = (value: number | null) => {
    dispatch(WManagerSettingsActions.setContainerPadding(value || 0));
  };

  const onChangeDefaultPadding = (value: number | null) => {
    dispatch(WManagerSettingsActions.setWorkspacePadding(value || 0));
  };

  return (
    <SettingsGroup>
      <div>
        <SettingsOption>
          <span>Gap between containers</span>
          <InputNumber value={containerPadding} onChange={onChangeDefaultGap} />
        </SettingsOption>
        <SettingsOption>
          <span>Workspace padding</span>
          <InputNumber value={workspacePadding} onChange={onChangeDefaultPadding} />
        </SettingsOption>
      </div>
      <SettingsSubGroup label="Workspace offset (margins)">
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
