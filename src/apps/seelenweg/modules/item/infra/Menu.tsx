import { SwItemType } from '../../../../shared/schemas/SeelenWegItems';
import { ItemType } from 'antd/es/menu/interface';
import { TFunction } from 'i18next';

import { store } from '../../shared/store/infra';

import { RootActions } from '../../shared/store/app';

import { SwItem } from '../../shared/store/domain';

export function getMenuForItem(t: TFunction, item: SwItem): ItemType[] {
  if (item.type === SwItemType.Media) {
    return [
      {
        key: 'remove',
        label: t('media_menu.remove'),
        onClick() {
          store.dispatch(RootActions.removeMediaModule());
        },
      },
    ];
  }

  if (item.type === SwItemType.Start) {
    return [
      {
        key: 'remove',
        label: t('start_menu.remove'),
        onClick() {
          store.dispatch(RootActions.removeStartModule());
        },
      },
    ];
  }

  return [];
}
