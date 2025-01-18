import { SeelenCommand } from '@seelen-ui/lib';
import { invoke } from '@tauri-apps/api/core';
import { ItemType } from 'antd/es/menu/interface';
import { TFunction } from 'i18next';

import { store } from '../shared/store/infra';
import { dialog } from 'src/apps/settings/modules/shared/tauri/infra';

import { RootActions } from '../shared/store/app';

import { Icon } from '../../../shared/components/Icon';

export function getSeelenWegMenu(t: TFunction, restrictedBar?: boolean): ItemType[] {
  if (!!restrictedBar) {
    return [
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
  } else {
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
}
