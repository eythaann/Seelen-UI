import { SeelenCommand, WorkspaceToolbarItemMode } from '@seelen-ui/lib';
import { WorkspaceToolbarItem } from '@seelen-ui/lib/types';
import { invoke } from '@tauri-apps/api/core';
import { Menu, Tooltip } from 'antd';
import { Reorder } from 'framer-motion';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from 'src/apps/seelenweg/components/BackgroundByLayers/infra';

import { CommonItemContextMenu } from '../item/app';
import { Selectors } from '../shared/store/app';
import { AnimatedDropdown } from 'src/apps/shared/components/AnimatedWrappers';
import { useWindowFocusChange } from 'src/apps/shared/hooks';

import { cx } from '../../../shared/styles';

interface Props {
  module: WorkspaceToolbarItem;
  onContextMenu?: (e: React.MouseEvent) => void;
}

function InnerWorkspacesModule({ module, ...rest }: Props) {
  const workspaces = useSelector(Selectors.workspaces);
  const activeWorkspace = useSelector(Selectors.activeWorkspace);

  const { mode } = module;

  console.log(rest);

  function onContextMenu(e: React.MouseEvent) {
    rest.onContextMenu?.(e);
    e.stopPropagation();
  }

  if (mode === WorkspaceToolbarItemMode.Dotted) {
    return (
      <Reorder.Item
        as="div"
        value={module}
        className="ft-bar-item"
        style={module.style}
        onContextMenu={onContextMenu}
      >
        <ul className="ft-bar-item-content workspaces">
          {workspaces.map((w, idx) => (
            <li
              key={w.id}
              onClick={() => invoke(SeelenCommand.SwitchWorkspace, { idx })}
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
    <Reorder.Item
      as="div"
      id={module.id}
      value={module}
      className="ft-bar-group"
      onContextMenu={onContextMenu}
    >
      {workspaces.map((w, idx) => {
        return (
          <Tooltip
            arrow={false}
            mouseLeaveDelay={0}
            classNames={{ root: 'ft-bar-item-tooltip' }}
            title={w.name || `Workspace ${idx + 1}`}
            key={w.id}
          >
            <div
              style={module.style}
              className={cx('ft-bar-item', {
                'ft-bar-item-clickable': true,
                'ft-bar-item-active': w.id === activeWorkspace,
              })}
              onClick={() => invoke(SeelenCommand.SwitchWorkspace, { idx })}
            >
              <div className="ft-bar-item-content">
                {mode === WorkspaceToolbarItemMode.Named
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

export function WorkspacesModule({ module }: Props) {
  const [openContextMenu, setOpenContextMenu] = useState(false);
  const workspaces = useSelector(Selectors.workspaces);

  const d = useDispatch();
  const { t } = useTranslation();

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenContextMenu(false);
    }
  });

  if (!workspaces.length) {
    return null;
  }

  return (
    <AnimatedDropdown
      animationDescription={{
        maxAnimationTimeMs: 500,
        openAnimationName: 'ft-bar-item-context-menu-open',
        closeAnimationName: 'ft-bar-item-context-menu-close',
      }}
      open={openContextMenu}
      onOpenChange={setOpenContextMenu}
      trigger={['contextMenu']}
      dropdownRender={() => (
        <BackgroundByLayersV2 className="ft-bar-item-context-menu-container">
          <Menu className="ft-bar-item-context-menu" items={CommonItemContextMenu(t, d, module)} />
        </BackgroundByLayersV2>
      )}
    >
      <InnerWorkspacesModule module={module} />
    </AnimatedDropdown>
  );
}
