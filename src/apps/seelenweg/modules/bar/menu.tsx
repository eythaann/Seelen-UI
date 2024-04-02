import { invoke } from '@tauri-apps/api/core';
import { Menu, MenuProps, Popover } from 'antd';

import { BackgroundByLayers } from '../../components/BackgrounByLayers/infra';
import { store } from '../shared/store/infra';

import { isRealPinned, isTemporalPinned, RootActions } from '../shared/store/app';

import { PinnedApp, PinnedAppSide } from '../shared/store/domain';

export function getMenuForItem(item: PinnedApp): MenuProps['items'] {
  const state = store.getState();
  const isPinned = isRealPinned(state, item);

  const pin = (side: PinnedAppSide) => {
    if (isTemporalPinned(item)) {
      store.dispatch(RootActions.pinApp({ app: item, side }));
    }
  };

  const menu: MenuProps['items'] = [];

  if (isPinned) {
    menu.push({
      label: 'Unpin',
      key: 'weg_unpin_app',
      onClick: () => {
        store.dispatch(RootActions.unPin(item));
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
            <>
              <BackgroundByLayers styles={state.theme.seelenweg.contextMenu.background} />
              <Menu
                style={state.theme.seelenweg.contextMenu.content}
                items={[
                  {
                    label: 'Pin to Left',
                    key: 'weg_pin_app_left',
                    onClick: () => pin(PinnedAppSide.LEFT),
                  },
                  {
                    label: 'Pin to Center',
                    key: 'weg_pin_app_center',
                    onClick: () => pin(PinnedAppSide.CENTER),
                  },
                  {
                    label: 'Pin to Right',
                    key: 'weg_pin_app_right',
                    onClick: () => pin(PinnedAppSide.RIGHT),
                  },
                ]}
              />
            </>
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
      label: 'Open File Location',
      key: 'weg_open_file_location',
      onClick: () => invoke('open_file_location', { path: item.exe }),
    },
  );

  if (isTemporalPinned(item)) {
    menu.push({
      label: item.opens.length > 1 ? 'Close All' : 'Close',
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
