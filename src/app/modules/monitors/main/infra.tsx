import { SettingsGroup, SettingsOption } from '../../../components/SettingsBox';
import { Button, Input, Modal, Select, Space } from 'antd';
import { useState } from 'react';

import { LayoutExamples } from '../layouts/infra';
import { WorkspaceConfig } from '../workspace/infra';
import cs from './infra.module.css';
import { AdvancedConfig } from './infra_advanced';

import { useAppSelector } from '../../shared/app/hooks';
import {
  GeneralSettingsSelectors,
  getMonitorSelector,
  RootSelectors,
} from '../../shared/app/selectors';
import { defaultOnNull } from '../../shared/app/utils';

export const MonitorConfig = ({ monitorIdx }: { monitorIdx: number }) => {
  const monitor = useAppSelector(getMonitorSelector(monitorIdx));

  if (!monitor) {
    return null;
  }

  const workspace = monitor.workspaces[monitor.edditingWorkspace] || monitor.workspaces[0]!;
  const LayoutExample = LayoutExamples[workspace.layout];

  const containerPadding = defaultOnNull(
    workspace.containerPadding,
    useAppSelector(GeneralSettingsSelectors.containerPadding),
  );

  const workspacePadding = defaultOnNull(
    workspace.workspacePadding,
    useAppSelector(GeneralSettingsSelectors.workspacePadding),
  );

  return (
    <div className={cs.config}>
      <div className={cs.monitor}>
        <div className={cs.border}>
          <div className={cs.screen}>
            <LayoutExample
              containerPadding={containerPadding}
              workspacePadding={workspacePadding}
            />
          </div>
        </div>
        <AdvancedConfig workspaceIdx={monitor.edditingWorkspace} monitorIdx={monitorIdx} />
        <Button type="primary" danger disabled={monitorIdx === 0}>
          Delete
        </Button>
        <Button type="primary">Insert</Button>
      </div>
      <SettingsGroup>
        <div>
          <div className={cs.title}>Monitor {monitorIdx + 1}</div>
          <Select
            className={cs.workspaceSelector}
            value={monitor.edditingWorkspace}
            dropdownRender={(menu) => (
              <>
                {menu}
                <hr />
                <Space>
                  <Input placeholder="New workspace" />
                  <Button type="primary">+</Button>
                </Space>
              </>
            )}
            options={monitor.workspaces.map((workspace, index) => ({
              label: workspace.name,
              value: index,
            }))}
          />
        </div>
        <WorkspaceConfig monitorIdx={monitorIdx} workspaceIdx={monitor.edditingWorkspace} />
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
