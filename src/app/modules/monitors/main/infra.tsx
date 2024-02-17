import { Select } from 'antd';

import { LayoutExamples } from '../layouts/infra';
import { WorkspaceConfig } from '../workspace/infra';
import cs from './infra.module.css';

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
      <div className={cs.border}>
        <div className={cs.screen}>
          <LayoutExample containerPadding={containerPadding} workspacePadding={workspacePadding} />
        </div>
      </div>
      <div>
        <Select
          value={monitor.edditingWorkspace}
          options={monitor.workspaces.map((workspace, index) => ({
            label: workspace.name,
            value: index,
          }))}
        />
        <WorkspaceConfig monitorIdx={monitorIdx} workspaceIdx={monitor.edditingWorkspace} />
      </div>
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
