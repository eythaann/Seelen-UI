import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Dropdown, Menu } from 'antd';
import { memo } from 'react';

import { OverflowTooltip } from 'src/apps/shared/components/OverflowTooltip';

import { StartMenuApp } from '../../shared/store/domain';

export const Item = memo(({ item, hidden }: { item: StartMenuApp; hidden: boolean }) => {
  const { label, icon, executionPath, path } = item;

  function onClick() {
    invoke('open_file', { path: executionPath });
    getCurrentWindow().hide();
  }

  const shortPath = executionPath.slice(executionPath.indexOf('\\Programs\\') + 10);

  return (
    <Dropdown
      trigger={['contextMenu']}
      dropdownRender={() => (
        <Menu
          items={[
            {
              label: 'Open File Location',
              key: 'open',
              onClick() {
                invoke('select_file_on_explorer', { path });
              },
            },
          ]}
        />
      )}
    >
      <button
        style={{ display: hidden ? 'none' : undefined }}
        className="launcher-item"
        onClick={onClick}
      >
        <img className="launcher-item-icon" src={convertFileSrc(icon)} alt={label} />
        <OverflowTooltip className="launcher-item-label" text={label} />
        <OverflowTooltip className="launcher-item-path" text={shortPath} />
      </button>
    </Dropdown>
  );
});
