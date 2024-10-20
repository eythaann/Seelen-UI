import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Dropdown, Menu } from 'antd';
import { memo } from 'react';
import { useTranslation } from 'react-i18next';
import { SeelenCommand } from 'seelen-core';

import { OverflowTooltip } from 'src/apps/shared/components/OverflowTooltip';

import { StartMenuApp } from '../../shared/store/domain';

export const Item = memo(({ item, hidden }: { item: StartMenuApp; hidden: boolean }) => {
  const { label, icon, path } = item;

  const { t } = useTranslation();

  function onClick() {
    invoke(SeelenCommand.OpenFile, { path });
    getCurrentWindow().hide();
  }

  const shortPath = path.slice(path.indexOf('\\Programs\\') + 10);

  return (
    <Dropdown
      trigger={['contextMenu']}
      dropdownRender={() => (
        <Menu
          items={[
            {
              label: t('item.pin'),
              key: 'pin',
              onClick() {
                invoke(SeelenCommand.WegPinItem, { path });
              },
            },
            {
              label: t('item.open_location'),
              key: 'open',
              onClick() {
                invoke(SeelenCommand.SelectFileOnExplorer, { path });
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
