import { createSlice } from '@reduxjs/toolkit';
import { invoke } from '@tauri-apps/api/core';
import { MenuProps } from 'antd';

import { store } from '../shared/store/infra';

import { isAppPinned, RootActions } from '../shared/store/app';

import { SeelenWegMode, SeelenWegState } from '../../../settings/modules/seelenweg/domain';
import { OpenApp, PinnedApp, PinnedAppSide } from '../shared/store/domain';

const initialState: SeelenWegState = {
  enabled: true,
  mode: SeelenWegMode.MIN_CONTENT,
  size: 40,
  zoomSize: 70,
  margin: 8,
  padding: 8,
  spaceBetweenItems: 8,
};

export const SeelenWegSlice = createSlice({
  name: 'seelenweg',
  initialState,
  reducers: {},
});

export function getMenuForItem(item: OpenApp | PinnedApp): MenuProps['items'] {
  const state = store.getState();
  const isPinned = isAppPinned(state, item);

  const pin = (side: PinnedAppSide) => {
    store.dispatch(RootActions.pinApp({ app: item as OpenApp, side }));
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
    menu.push(
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
    );
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
    {
      label: 'Close',
      key: 'weg_close_app',
    },
  );

  return menu;
}
