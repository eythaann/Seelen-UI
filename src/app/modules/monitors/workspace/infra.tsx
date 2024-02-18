import { SettingsGroup, SettingsOption } from '../../../components/SettingsBox';
import { InputNumber, Select } from 'antd';

import cs from './infra.module.css';

import { useAppSelector } from '../../shared/app/hooks';
import { getWorkspaceSelector } from '../../shared/app/selectors';
import { getContainerPaddingSelector, getWorkspacePaddingSelector } from './app';

import { Layout } from '../layouts/domain';

interface Props {
  monitorIdx: number;
  workspaceIdx: number;
}

export const WorkspaceConfig = ({ monitorIdx, workspaceIdx }: Props) => {
  const workspace = useAppSelector(getWorkspaceSelector(workspaceIdx, monitorIdx));

  const workspacePadding = useAppSelector(getWorkspacePaddingSelector(workspaceIdx, monitorIdx));
  const containerPadding = useAppSelector(getContainerPaddingSelector(workspaceIdx, monitorIdx));

  if (!workspace) {
    return null;
  }

  return (
    <div className={cs.workspaceConfig}>
      <SettingsOption>
        <span>padding</span>
        <InputNumber value={workspace.workspacePadding} placeholder="Global" />
      </SettingsOption>
      <SettingsOption>
        <span>gap</span>
        <InputNumber value={workspace.containerPadding} placeholder="Global" />
      </SettingsOption>
      <SettingsOption>
        <span>layout</span>
        <Select
          value={workspace.layout}
          options={Object.values(Layout).map((op) => ({
            label: op,
          }))}
        />
      </SettingsOption>
    </div>
  );
};
