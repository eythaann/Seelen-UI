import { SeelenCommand } from '@seelen-ui/lib';
import { path } from '@tauri-apps/api';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Dropdown, Menu } from 'antd';
import { memo } from 'react';
import { useTranslation } from 'react-i18next';

import { OverflowTooltip } from 'src/apps/shared/components/OverflowTooltip';
import { useIcon } from 'src/apps/shared/hooks';

import { StartMenuApp } from '../../shared/store/domain';

const MISSING_ICON_SRC = convertFileSrc(await path.resolveResource('static/icons/missing.png'));

export const Item = memo(({ item, hidden }: { item: StartMenuApp; hidden: boolean }) => {
  const { path, umid } = item;

  const { t } = useTranslation();

  const icon = useIcon({ path, umid: umid });

  function onClick() {
    invoke(SeelenCommand.OpenFile, { path });
    getCurrentWindow().hide();
  }

  const parts = path.split('\\');
  const filename = parts.at(-1);

  const shortPath = path.slice(path.indexOf('\\Programs\\') + 10);
  const displayName = filename?.slice(0, filename.lastIndexOf('.')) || filename || '';

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
        <img className="launcher-item-icon" src={icon || MISSING_ICON_SRC} />
        <OverflowTooltip className="launcher-item-label" text={displayName} />
        <OverflowTooltip className="launcher-item-path" text={shortPath} />
      </button>
    </Dropdown>
  );
});
