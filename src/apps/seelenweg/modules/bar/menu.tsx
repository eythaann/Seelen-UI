import { savePinnedItems } from '../shared/store/storeApi';
import { invoke } from '@tauri-apps/api/core';
import { Menu, MenuProps, Popover } from 'antd';

import { BackgroundByLayers } from '../../components/BackgrounByLayers/infra';
import { store } from '../shared/store/infra';

import { isRealPinned, isTemporalPinned, RootActions } from '../shared/store/app';

import { App, AppsSides } from '../shared/store/domain';

export function getMenuForItem(item: App): MenuProps['items'] {
  const state = store.getState();
  const isPinned = isRealPinned(item);

  const pin = (side: AppsSides) => {
    if (isTemporalPinned(item)) {
      store.dispatch(RootActions.pinApp({ app: item, side }));
      savePinnedItems(store.getState());
    }
  };

  const menu: MenuProps['items'] = [];

  if (isPinned) {
    menu.push({
      label: 'Unpin',
      key: 'weg_unpin_app',
      onClick: () => {
        store.dispatch(RootActions.unPin(item));
        savePinnedItems(store.getState());
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
            <div className="weg-context-menu-container">
              <BackgroundByLayers prefix="menu" layers={state.themeLayers.weg.contextMenu.bg} />
              <Menu
                className="weg-context-menu"
                items={[
                  {
                    label: 'Pin to left',
                    key: 'weg_pin_app_left',
                    onClick: () => pin(AppsSides.LEFT),
                  },
                  {
                    label: 'Pin to center',
                    key: 'weg_pin_app_center',
                    onClick: () => pin(AppsSides.CENTER),
                  },
                  {
                    label: 'Pin to right',
                    key: 'weg_pin_app_right',
                    onClick: () => pin(AppsSides.RIGHT),
                  },
                ]}
              />
            </div>
          }
        >
          <div style={{ width: '100%', height: '100%', margin: '-10px', padding: '10px' }}>Pin</div>
        </Popover>
      ),
    });
  }

  menu.push(
    {
      type: 'divider',
    },
    {
      label: 'Open file location',
      key: 'weg_select_file_on_explorer',
      onClick: () => invoke('select_file_on_explorer', { path: item.exe }),
    },
    {
      label: 'Run as administrator',
      key: 'weg_runas',
      onClick: () => invoke('run_as_admin', { path: item.execution_path }),
    },
  );

  if (item.opens.length) {
    menu.push(
      {
        label: 'Copy handles',
        key: 'weg_copy_hwnd',
        onClick: () => navigator.clipboard.writeText(JSON.stringify(item.opens.map((hwnd) => hwnd.toString(16)))),
      },
      {
        label: item.opens.length > 1 ? 'Close all' : 'Close',
        key: 'weg_close_app',
        onClick() {
          item.opens.forEach((hwnd) => {
            invoke('weg_close_app', { hwnd });
          });
        },
        danger: true,
      });
  }

  return menu;
}
