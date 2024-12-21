import { SeelenCommand } from '@seelen-ui/lib';
import { invoke } from '@tauri-apps/api/core';
import { Menu, MenuProps, Popover } from 'antd';
import { ItemType } from 'antd/es/menu/interface';
import { TFunction } from 'i18next';

import { BackgroundByLayersV2 } from '../../components/BackgroundByLayers/infra';
import { store } from '../shared/store/infra';
import { dialog } from 'src/apps/settings/modules/shared/tauri/infra';

import { isPinnedApp, isTemporalApp, RootActions } from '../shared/store/app';

import { AppsSides, PinnedWegItem, TemporalWegItem } from '../shared/store/domain';

import { Icon } from '../../../shared/components/Icon';
import { savePinnedItems } from '../shared/store/storeApi';

export function getSeelenWegMenu(t: TFunction): ItemType[] {
  return [
    {
      key: 'add-media-module',
      label: t('taskbar_menu.media'),
      icon: <Icon iconName="PiMusicNotesPlusFill" />,
      onClick() {
        store.dispatch(RootActions.addMediaModule());
      },
    },
    {
      key: 'add-start-module',
      label: t('taskbar_menu.start'),
      icon: <Icon iconName="SiWindows" size={14} />,
      onClick() {
        store.dispatch(RootActions.addStartModule());
      },
    },
    {
      type: 'divider',
    },
    {
      key: 'add-item',
      label: t('taskbar_menu.add_file'),
      icon: <Icon iconName="RiFileAddLine" />,
      async onClick() {
        const files = await dialog.open({
          title: t('taskbar_menu.add_file'),
          multiple: true,
          filters: [
            { name: 'lnk', extensions: ['lnk'] },
            { name: '*', extensions: ['*'] },
          ],
        });
        for (const path of files || []) {
          await invoke(SeelenCommand.WegPinItem, { path });
        }
      },
    },
    {
      key: 'add-folder',
      label: t('taskbar_menu.add_folder'),
      icon: <Icon iconName="RiFolderAddLine" />,
      async onClick() {
        const folder = await dialog.open({
          title: t('taskbar_menu.add_folder'),
          directory: true,
        });
        if (folder) {
          await invoke(SeelenCommand.WegPinItem, { path: folder });
        }
      },
    },
    {
      type: 'divider',
    },
    {
      key: 'task_manager',
      label: t('taskbar_menu.task_manager'),
      icon: <Icon iconName="PiChartLineFill" />,
      onClick() {
        invoke(SeelenCommand.OpenFile, { path: 'C:\\Windows\\System32\\Taskmgr.exe' });
      },
    },
    {
      key: 'settings',
      label: t('taskbar_menu.settings'),
      icon: <Icon iconName="RiSettings4Fill" />,
      onClick() {
        invoke(SeelenCommand.ShowAppSettings);
      },
    },
  ];
}

export function getMenuForItem(
  t: TFunction,
  item: PinnedWegItem | TemporalWegItem,
  devTools: boolean,
): ItemType[] {
  const isPinned = isPinnedApp(item);

  const pin = (side: AppsSides) => {
    if (isTemporalApp(item)) {
      store.dispatch(RootActions.pinApp({ app: item, side }));
      savePinnedItems();
    }
  };

  const menu: MenuProps['items'] = [];

  if (isPinned) {
    menu.push({
      label: t('app_menu.unpin'),
      key: 'weg_unpin_app',
      icon: <Icon iconName="RiUnpinLine" />,
      onClick: () => {
        store.dispatch(RootActions.unPinApp(item));
        savePinnedItems();
      },
    });
  } else {
    menu.push({
      key: 'weg_pin_app',
      icon: <Icon iconName="RiPushpinLine" />,
      label: (
        <Popover
          trigger={['hover']}
          placement="rightBottom"
          arrow={false}
          content={
            <BackgroundByLayersV2 className="weg-context-menu-container" prefix="menu">
              <Menu
                className="weg-context-menu"
                items={[
                  {
                    key: 'weg_pin_app_left',
                    label: t('app_menu.pin_to_left'),
                    icon: <Icon iconName="RxPinLeft" />,
                    onClick: () => pin(AppsSides.Left),
                  },
                  {
                    key: 'weg_pin_app_center',
                    label: t('app_menu.pin_to_center'),
                    icon: <Icon iconName="RiPushpinLine" />,
                    onClick: () => pin(AppsSides.Center),
                  },
                  {
                    key: 'weg_pin_app_right',
                    label: t('app_menu.pin_to_right'),
                    icon: <Icon iconName="RxPinRight" />,
                    onClick: () => pin(AppsSides.Right),
                  },
                ]}
              />
            </BackgroundByLayersV2>
          }
        >
          <div style={{ width: '100%', height: '100%', margin: '-10px', padding: '10px' }}>
            {t('app_menu.pin')}
          </div>
        </Popover>
      ),
    });
  }

  menu.push(
    {
      type: 'divider',
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
      onClick: () => invoke(SeelenCommand.RunAsAdmin, { path: item.execution_command }),
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
        navigator.clipboard.writeText(JSON.stringify(item.windows.map((window) => window.handle.toString(16)))),
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
