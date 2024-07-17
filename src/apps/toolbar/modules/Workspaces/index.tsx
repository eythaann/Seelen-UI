import { WorkspacesTM, WorkspaceTMMode } from '../../../shared/schemas/Placeholders';
import { cx } from '../../../shared/styles';
import { invoke } from '@tauri-apps/api/core';
import { Tooltip } from 'antd';
import { Reorder } from 'framer-motion';
import { useSelector } from 'react-redux';

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
      <Reorder.Item as="div" value={module} className="ft-bar-item" style={module.style}>
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
      </Reorder.Item>
    );
  }

  return (
    <Reorder.Item as="div" id={module.id} value={module}>
      {workspaces.map((name, idx) => {
        return (
          <Tooltip
            arrow={false}
            mouseLeaveDelay={0}
            overlayClassName="ft-bar-item-tooltip"
            title={name}
            key={name}
          >
            <div
              style={module.style}
              className={cx('ft-bar-item', {
                'ft-bar-item-clickable': true,
                'ft-bar-item-active': idx === activeWorkspace,
              })}
              onClick={() => invoke('switch_workspace', { idx })}
            >
              <div className="ft-bar-item-content">
                {mode === WorkspaceTMMode.Named ? `${name}` : `${idx + 1}`}
              </div>
            </div>
          </Tooltip>
        );
      })}
    </Reorder.Item>
  );
}
