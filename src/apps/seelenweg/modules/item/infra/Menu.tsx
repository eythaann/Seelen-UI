import { SeelenCommand, WegItemType } from '@seelen-ui/lib';
import { invoke } from '@tauri-apps/api/core';
import { ItemType } from 'antd/es/menu/interface';
import { TFunction } from 'i18next';

import { store } from '../../shared/store/infra';

import { RootActions } from '../../shared/store/app';

import { SwItem } from '../../shared/store/domain';

import { Icon } from '../../../../shared/components/Icon';
import { savePinnedItems } from '../../shared/store/storeApi';

export function getMenuForItem(t: TFunction, item: SwItem): ItemType[] {
  if (item.type === WegItemType.Media) {
    return [
      {
        key: 'remove',
        label: t('media_menu.remove'),
        icon: <Icon iconName="CgExtensionRemove" />,
        onClick() {
          store.dispatch(RootActions.removeMediaModule());
        },
      },
    ];
  }

  if (item.type === WegItemType.StartMenu) {
    return [
      {
        key: 'remove',
        label: t('start_menu.remove'),
        icon: <Icon iconName="CgExtensionRemove" />,
        onClick() {
          store.dispatch(RootActions.removeStartModule());
        },
      },
    ];
  }

  if (item.type === WegItemType.Pinned) {
    return [
      {
        key: 'remove',
        label: t('app_menu.unpin'),
        icon: <Icon iconName="RiUnpinLine" />,
        onClick() {
          store.dispatch(RootActions.unpin(item));
          savePinnedItems();
        },
      },
      {
        type: 'divider',
      },
      {
        key: 'weg_select_file_on_explorer',
        label: t('app_menu.open_file_location'),
        icon: <Icon iconName="MdOutlineMyLocation" />,
        onClick: () => invoke(SeelenCommand.SelectFileOnExplorer, { path: item.path }),
      },
    ];
  }

  return [];
}
