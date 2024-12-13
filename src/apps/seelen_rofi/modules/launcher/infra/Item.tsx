import { SeelenCommand } from '@seelen-ui/lib';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Menu } from 'antd';
import { memo } from 'react';
import { useTranslation } from 'react-i18next';

import { StartMenuApp } from '../../shared/store/domain';

import { AnimatedDropdown } from '../../../../shared/components/AnimatedWrappers';
import { OverflowTooltip } from '../../../../shared/components/OverflowTooltip';

export const Item = memo(({ item, hidden }: { item: StartMenuApp; hidden: boolean }) => {
  const { label, icon, path } = item;

  const { t } = useTranslation();

  function onClick() {
    invoke(SeelenCommand.OpenFile, { path });
    getCurrentWindow().hide();
  }

  const shortPath = path.slice(path.indexOf('\\Programs\\') + 10);

  return (
    <AnimatedDropdown
      animationDescription={{
        maxAnimationTimeMs: 500,
        openAnimationName: 'launcher-item-context-menu-open',
        closeAnimationName: 'launcher-item-context-menu-close',
      }}
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
    </AnimatedDropdown>
  );
});
