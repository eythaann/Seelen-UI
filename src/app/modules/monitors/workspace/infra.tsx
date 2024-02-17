import { useAppSelector } from '../../shared/app/hooks';
import { getWorkspaceSelector } from '../../shared/app/selectors';
import { getContainerPaddingSelector, getWorkspacePaddingSelector } from './app';

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

  return <div>
    padding: {workspacePadding}
    gap: {containerPadding}
    layout: {workspace.layout}
  </div>;
};