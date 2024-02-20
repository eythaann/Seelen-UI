import { SettingsOption } from '../../../components/SettingsBox';
import { InputNumber, Select } from 'antd';
import { useDispatch } from 'react-redux';

import cs from './infra.module.css';

import { useAppSelector } from '../../shared/app/hooks';
import { getWorkspaceSelector } from '../../shared/app/selectors';
import { OptionsFromEnum } from '../../shared/app/utils';
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
    dispatch(MonitorsActions.updateWorkspace({ monitorIdx, workspaceIdx, key: 'containerPadding', value }));
  };

  const onChangePadding = (value: number | null) => {
    dispatch(MonitorsActions.updateWorkspace({ monitorIdx, workspaceIdx, key: 'workspacePadding', value }));
  };

  return (
    <div className={cs.workspaceConfig}>
      <SettingsOption>
        <span>padding</span>
        <InputNumber value={workspace.workspacePadding} placeholder="Global" onChange={onChangePadding} />
      </SettingsOption>
      <SettingsOption>
        <span>gap</span>
        <InputNumber value={workspace.containerPadding} placeholder="Global" onChange={onChangeGap} />
      </SettingsOption>
      <SettingsOption>
        <span>layout</span>
        <Select value={workspace.layout} options={OptionsFromEnum(Layout)} onChange={onSelectLayout} />
      </SettingsOption>
    </div>
  );
};
