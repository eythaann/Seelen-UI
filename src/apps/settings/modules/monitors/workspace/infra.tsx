import { SettingsOption } from '../../../components/SettingsBox';
import { InputNumber, Select } from 'antd';
import { useDispatch } from 'react-redux';

import { useAppSelector } from '../../shared/utils/infra';
import cs from './infra.module.css';

import { getWorkspaceSelector } from '../../shared/store/app/selectors';
import { OptionsFromEnum } from '../../shared/utils/app';
import { MonitorsActions } from '../main/app';

import { Layout } from '../layouts/domain';

interface Props {
  monitorIdx: number;
  workspaceIdx: number;
}

export const WorkspaceConfig = ({ monitorIdx, workspaceIdx }: Props) => {
  const workspace = useAppSelector(getWorkspaceSelector(workspaceIdx, monitorIdx));

  const dispatch = useDispatch();

  if (!workspace) {
    return null;
  }

  const onSelectLayout = (layout: Layout) => {
    dispatch(MonitorsActions.updateWorkspace({ monitorIdx, workspaceIdx, key: 'layout', value: layout }));
  };

  const onChangeGap = (value: number | null) => {
    dispatch(MonitorsActions.updateWorkspace({ monitorIdx, workspaceIdx, key: 'gap', value }));
  };

  const onChangePadding = (value: number | null) => {
    dispatch(MonitorsActions.updateWorkspace({ monitorIdx, workspaceIdx, key: 'padding', value }));
  };

  return (
    <div className={cs.workspaceConfig}>
      <SettingsOption>
        <span>padding</span>
        <InputNumber value={workspace.padding} placeholder="Global" onChange={onChangePadding} />
      </SettingsOption>
      <SettingsOption>
        <span>gap</span>
        <InputNumber value={workspace.gap} placeholder="Global" onChange={onChangeGap} />
      </SettingsOption>
      <SettingsOption>
        <span>layout</span>
        <Select value={workspace.layout as any} options={OptionsFromEnum(Layout)} onChange={onSelectLayout} />
      </SettingsOption>
    </div>
  );
};
