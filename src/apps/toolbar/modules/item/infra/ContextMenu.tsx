import { ToolbarItem } from '@seelen-ui/lib/types';
import { Icon } from '@shared/components/Icon';
import { TFunction } from 'i18next';
import { Dispatch } from 'redux';

import { SaveToolbarItems } from '../../main/application';
import { RootActions } from '../../shared/store/app';

export function CommonItemContextMenu(t: TFunction, d: Dispatch, item: Omit<ToolbarItem, 'type'>) {
  return [
    {
      key: 'remove',
      label: t('context_menu.remove'),
      icon: <Icon iconName="CgExtensionRemove" />,
      className: 'ft-bar-item-context-menu-item',
      onClick() {
        d(RootActions.removeItem(item.id));
        SaveToolbarItems()?.catch(console.error);
      },
    },
  ];
}
