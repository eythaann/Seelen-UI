import { SeelenCommand } from '@seelen-ui/lib';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { MenuProps } from 'antd';
import { ItemType } from 'antd/es/menu/interface';
import { TFunction } from 'i18next';

import { store } from '../../shared/store/infra';
import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { isPinnedApp, RootActions } from '../../shared/store/app';
import { parseCommand } from 'src/apps/shared/Command';
import { useIcon } from 'src/apps/shared/hooks';

import { PinnedWegItem, TemporalWegItem } from '../../shared/store/domain';

import { Icon } from '../../../../shared/components/Icon';

export function getUserApplicationContextMenu(
  t: TFunction,
  item: PinnedWegItem | TemporalWegItem,
  devTools: boolean,
): ItemType[] {
  const isPinned = isPinnedApp(item);

  const menu: MenuProps['items'] = [];

  if (!item.pinDisabled) {
    if (isPinned) {
      menu.push({
        label: t('app_menu.unpin'),
        key: 'weg_unpin_app',
        icon: <Icon iconName="RiUnpinLine" />,
        onClick: () => {
          if (item.windows.length) {
            store.dispatch(RootActions.unPinApp(item.id));
          } else {
            store.dispatch(RootActions.remove(item.id));
          }
        },
      });
    } else {
      menu.push({
        key: 'weg_pin_app',
        icon: <Icon iconName="RiPushpinLine" />,
        label: (
          <div style={{ width: '100%', height: '100%', margin: '-10px', padding: '10px' }}>
            {t('app_menu.pin')}
          </div>
        ),
        onClick: () => store.dispatch(RootActions.pinApp(item.id)),
      });
    }

    menu.push(
      {
        type: 'divider',
      },
    );
  }

  const iconSrc =
    useIcon({
      path: item.path,
      umid: item.umid,
    }) || convertFileSrc(LAZY_CONSTANTS.MISSING_ICON_PATH);

  menu.push(
    {
      key: 'weg_run_new',
      label: item.displayName,
      icon: <img className="weg-context-menu-item-icon" src={iconSrc} />,
      onClick: () => {
        const { program, args } = parseCommand(item.relaunchCommand);
        invoke(SeelenCommand.Run, { program, args });
      },
    },
    {
      key: 'weg_select_file_on_explorer',
      label: t('app_menu.open_file_location'),
      icon: <Icon iconName="MdOutlineMyLocation" />,
      onClick: () => invoke(SeelenCommand.SelectFileOnExplorer, { path: item.path }),
    },
    {
      key: 'weg_runas',
      label: t('app_menu.run_as'),
      icon: <Icon iconName="MdOutlineAdminPanelSettings" />,
      onClick: () => {
        const { program, args } = parseCommand(item.relaunchCommand);
        invoke(SeelenCommand.RunAsAdmin, { program, args });
      },
    },
  );

  if (!item.windows.length) {
    return menu;
  }

  if (devTools) {
    menu.push({
      key: 'weg_copy_hwnd',
      label: t('app_menu.copy_handles'),
      icon: <Icon iconName="AiOutlineCopy" />,
      onClick: () =>
        navigator.clipboard.writeText(
          JSON.stringify(item.windows.map((window) => window.handle.toString(16))),
        ),
    });
  }

  menu.push({
    key: 'weg_close_app',
    label: item.windows.length > 1 ? t('app_menu.close_multiple') : t('app_menu.close'),
    icon: <Icon iconName="BiWindowClose" />,
    onClick() {
      item.windows.forEach((window) => {
        invoke(SeelenCommand.WegCloseApp, { hwnd: window.handle });
      });
    },
    danger: true,
  });

  if (devTools) {
    menu.push({
      key: 'weg_kill_app',
      label: item.windows.length > 1 ? t('app_menu.kill_multiple') : t('app_menu.kill'),
      icon: <Icon iconName="MdOutlineDangerous" size={18} />,
      onClick() {
        item.windows.forEach((window) => {
          // todo replace by enum
          invoke(SeelenCommand.WegKillApp, { hwnd: window.handle });
        });
      },
      danger: true,
    });
  }

  return menu;
}
