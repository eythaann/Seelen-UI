import { invoke } from '@tauri-apps/api/core';
import { Menu, MenuProps, Popover } from 'antd';
import { ItemType } from 'antd/es/menu/interface';
import { TFunction } from 'i18next';
import { SeelenCommand } from 'seelen-core';

import { BackgroundByLayersV2 } from '../../components/BackgroundByLayers/infra';
import { store } from '../shared/store/infra';

import { isPinnedApp, isTemporalApp, RootActions } from '../shared/store/app';

import { AppsSides, ExtendedPinnedWegItem, ExtendedTemporalWegItem } from '../shared/store/domain';

import { savePinnedItems } from '../shared/store/storeApi';

export function getSeelenWegMenu(t: TFunction): ItemType[] {
  return [
    {
      key: 'add-media-module',
      label: t('taskbar_menu.media'),
      onClick() {
        store.dispatch(RootActions.addMediaModule());
      },
    },
    {
      key: 'add-start-module',
      label: t('taskbar_menu.start'),
      onClick() {
        store.dispatch(RootActions.addStartModule());
      },
    },
    {
      key: 'settings',
      label: t('taskbar_menu.settings'),
      onClick() {
        invoke(SeelenCommand.ShowAppSettings);
      },
    },
  ];
}

export function getMenuForItem(t: TFunction, item: ExtendedPinnedWegItem | ExtendedTemporalWegItem): ItemType[] {
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
      onClick: () => {
        store.dispatch(RootActions.unPin(item));
        savePinnedItems();
      },
    });
  } else {
    menu.push({
      key: 'weg_pin_app',
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
                    onClick: () => pin(AppsSides.Left),
                  },
                  {
                    key: 'weg_pin_app_center',
                    label: t('app_menu.pin_to_center'),
                    onClick: () => pin(AppsSides.Center),
                  },
                  {
                    key: 'weg_pin_app_right',
                    label: t('app_menu.pin_to_right'),
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
      onClick: () => invoke(SeelenCommand.SelectFileOnExplorer, { path: item.exe }),
    },
    {
      key: 'weg_runas',
      label: t('app_menu.run_as'),
      onClick: () => invoke(SeelenCommand.RunAsAdmin, { path: item.execution_path }),
    },
  );

  if (item.opens.length) {
    menu.push(
      {
        key: 'weg_copy_hwnd',
        label: t('app_menu.copy_handles'),
        onClick: () =>
          navigator.clipboard.writeText(
            JSON.stringify(item.opens.map((hwnd) => hwnd.toString(16))),
          ),
      },
      {
        key: 'weg_close_app',
        label: item.opens.length > 1 ? t('app_menu.close_multiple') : t('app_menu.close'),
        onClick() {
          item.opens.forEach((hwnd) => {
            invoke(SeelenCommand.WegCloseApp, { hwnd });
          });
        },
        danger: true,
      },
    );
  }

  return menu;
}
