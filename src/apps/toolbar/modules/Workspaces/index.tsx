import { WorkspacesTM, WorkspaceTMMode } from '../../../utils/schemas/Placeholders';
import { cx } from '../../../utils/styles';
import { invoke } from '@tauri-apps/api/core';
import { useSelector } from 'react-redux';

import { Item } from '../item/infra';

import { Selectors } from '../shared/store/app';

interface Props {
  module: WorkspacesTM;
}

export function WorkspacesModule({ module }: Props) {
  const workspaces = useSelector(Selectors.workspaces);
  const activeWorkspace = useSelector(Selectors.activeWorkspace);

  const { mode } = module;

  if (mode === WorkspaceTMMode.Dotted) {
    return (
      <div className="ft-bar-item">
        <ul className="ft-bar-item-content workspaces">
          {workspaces.map((_, idx) => (
            <li
              key={idx}
              onClick={() => invoke('switch_workspace', { idx })}
              className={cx('workspace-dot', {
                'workspace-dot-active': idx === activeWorkspace,
              })}
            />
          ))}
        </ul>
      </div>
    );
  }

  return workspaces.map((name, idx) => {
    return (
      <Item
        key={name}
        active={idx === activeWorkspace}
        module={{
          ...module,
          onClick: `switch-workspace -> ${idx}`,
          template: mode === WorkspaceTMMode.Named ? `"${name}"` : `"${idx + 1}"`,
        }}
      />
    );
  });
}
