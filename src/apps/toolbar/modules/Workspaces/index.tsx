import { invoke } from '@tauri-apps/api/core';
import { Tooltip } from 'antd';
import { Reorder } from 'framer-motion';
import { useSelector } from 'react-redux';

import { Selectors } from '../shared/store/app';

import { WorkspacesTM, WorkspaceTMMode } from '../../../shared/schemas/Placeholders';
import { cx } from '../../../shared/styles';

interface Props {
  module: WorkspacesTM;
}

export function WorkspacesModule({ module }: Props) {
  const workspaces = useSelector(Selectors.workspaces);
  const activeWorkspace = useSelector(Selectors.activeWorkspace);

  const { mode } = module;

  if (workspaces.length === 0) {
    return null;
  }

  if (mode === WorkspaceTMMode.Dotted) {
    return (
      <Reorder.Item as="div" value={module} className="ft-bar-item" style={module.style}>
        <ul className="ft-bar-item-content workspaces">
          {workspaces.map((w, idx) => (
            <li
              key={w.id}
              onClick={() => invoke('switch_workspace', { idx })}
              className={cx('workspace-dot', {
                'workspace-dot-active': w.id === activeWorkspace,
              })}
            />
          ))}
        </ul>
      </Reorder.Item>
    );
  }

  return (
    <Reorder.Item as="div" id={module.id} value={module} className="ft-bar-group">
      {workspaces.map((w, idx) => {
        return (
          <Tooltip
            arrow={false}
            mouseLeaveDelay={0}
            overlayClassName="ft-bar-item-tooltip"
            title={w.name || `Workspace ${idx + 1}`}
            key={w.id}
          >
            <div
              style={module.style}
              className={cx('ft-bar-item', {
                'ft-bar-item-clickable': true,
                'ft-bar-item-active': w.id === activeWorkspace,
              })}
              onClick={() => invoke('switch_workspace', { idx })}
            >
              <div className="ft-bar-item-content">
                {mode === WorkspaceTMMode.Named
                  ? `${w.name || `Workspace ${idx + 1}`}`
                  : `${idx + 1}`}
              </div>
            </div>
          </Tooltip>
        );
      })}
    </Reorder.Item>
  );
}
