import { SeelenCommand, WegItemType } from '@seelen-ui/lib';
import { dialog } from '@seelen-ui/lib/tauri';
import { Icon } from '@shared/components/Icon';
import { invoke } from '@tauri-apps/api/core';
import { ItemType } from 'antd/es/menu/interface';
import { TFunction } from 'i18next';

import { $dock_state, $dock_state_actions } from '../shared/state/items';

export function getSeelenWegMenu(t: TFunction): ItemType[] {
  const isRestrictedBar =
    $dock_state.value.items.filter((c) => c.type !== WegItemType.Separator).length > 0 &&
    $dock_state.value.items.every((item) => item.type === WegItemType.Temporal && item.pinDisabled);

  if (!!isRestrictedBar) {
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
  }

  return [
    {
      key: 'add-media-module',
      label: t('taskbar_menu.media'),
      icon: <Icon iconName="PiMusicNotesPlusFill" />,
      onClick() {
        $dock_state_actions.addMediaModule();
      },
    },
    {
      key: 'add-start-module',
      label: t('taskbar_menu.start'),
      icon: <Icon iconName="BsWindows" size={14} />,
      onClick() {
        $dock_state_actions.addStartModule();
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
      key: 'reoder',
      icon: <Icon iconName={$dock_state.value.isReorderDisabled ? 'CgLockUnlock' : 'CgLock'} />,
      label: t(
        $dock_state.value.isReorderDisabled
          ? 'context_menu.reorder_enable'
          : 'context_menu.reorder_disable',
      ),
      onClick() {
        $dock_state.value = {
          ...$dock_state.value,
          isReorderDisabled: !$dock_state.value.isReorderDisabled,
        };
      },
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
