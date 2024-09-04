import { SettingsGroup } from '../../../components/SettingsBox';
import { Button, Input, Select, Space } from 'antd';
import { useState } from 'react';
import { useDispatch } from 'react-redux';

import { useAppSelector } from '../../shared/utils/infra';
import { LayoutExamples } from '../layouts/infra';
import { WorkspaceConfig } from '../workspace/infra';
import cs from './infra.module.css';
import { AdvancedConfig } from './infra_advanced';

import { getMonitorSelector, RootSelectors, SeelenWmSelectors } from '../../shared/store/app/selectors';
import { defaultOnNull } from '../../shared/utils/app';
import { MonitorsActions } from './app';

export const MonitorConfig = ({ monitorIdx }: { monitorIdx: number }) => {
  const [newWorkspaceName, setNewWorkspaceName] = useState('');
  const monitor = useAppSelector(getMonitorSelector(monitorIdx));

  const dispatch = useDispatch();

  if (!monitor) {
    return null;
  }

  const workspace = monitor.workspaces[monitor.editingWorkspace || 0]!;
  const LayoutExample = LayoutExamples[workspace.layout];

  const containerPadding = defaultOnNull(
    workspace.gap,
    useAppSelector(SeelenWmSelectors.workspaceGap),
  );

  const workspacePadding = defaultOnNull(
    workspace.padding,
    useAppSelector(SeelenWmSelectors.workspacePadding),
  );

  const onDelete = () => {
    dispatch(MonitorsActions.delete(monitorIdx));
  };

  const onInsert = () => {
    dispatch(MonitorsActions.insert(monitorIdx + 1));
  };

  const onChangeWorkspace = (workspaceIdx: number) => {
    dispatch(MonitorsActions.changeEditingWorkspace({ monitorIdx, workspaceIdx }));
  };

  const onChangeNewWorkspaceName = (event: React.ChangeEvent<HTMLInputElement>) => {
    setNewWorkspaceName(event.target.value);
  };
  const onAddWorkspace = () => {
    dispatch(MonitorsActions.newWorkspace({ monitorIdx, name: newWorkspaceName }));
    setNewWorkspaceName('');
  };

  return (
    <div className={cs.config}>
      <div className={cs.monitor}>
        <div className={cs.border}>
          <div className={cs.screen}>
            {LayoutExample && <LayoutExample containerPadding={containerPadding} workspacePadding={workspacePadding} />}
          </div>
        </div>
        <AdvancedConfig workspaceIdx={monitor.editingWorkspace || 0} monitorIdx={monitorIdx} />
        <Button type="primary" danger disabled={monitorIdx === 0} onClick={onDelete}>
          Delete
        </Button>
        <Button type="primary" onClick={onInsert}>
          Insert
        </Button>
      </div>
      <SettingsGroup>
        <div>
          <div className={cs.title}>Monitor {monitorIdx + 1}</div>
          <Select
            className={cs.workspaceSelector}
            value={monitor.editingWorkspace}
            dropdownRender={(menu) => (
              <>
                {menu}
                <hr />
                <Space>
                  <Input value={newWorkspaceName} placeholder="New workspace" onChange={onChangeNewWorkspaceName} />
                  <Button type="primary" onClick={onAddWorkspace}>
                    +
                  </Button>
                </Space>
              </>
            )}
            options={monitor.workspaces.map((workspace, index) => ({
              label: workspace.name,
              value: index,
            }))}
            onChange={onChangeWorkspace}
          />
        </div>
        <WorkspaceConfig monitorIdx={monitorIdx} workspaceIdx={monitor.editingWorkspace || 0} />
      </SettingsGroup>
    </div>
  );
};

export function Monitors() {
  const monitors = useAppSelector(RootSelectors.monitors);

  return (
    <div className={cs.monitors}>
      {monitors.map((_, index) => (
        <MonitorConfig key={index} monitorIdx={index} />
      ))}
    </div>
  );
}
