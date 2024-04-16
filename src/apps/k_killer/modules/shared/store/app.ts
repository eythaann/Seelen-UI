import { toPhysicalPixels } from '../../../../utils';
import { StateBuilder } from '../../../../utils/StateBuilder';
import { createSlice, current, PayloadAction } from '@reduxjs/toolkit';
import { invoke } from '@tauri-apps/api/core';
import { cloneDeep } from 'lodash';

import { addHandleToLayout, removeHandleFromLayout } from '../../layout/app';

import { BoxSubType, BoxType, Layout } from '../../layout/domain';
import { DesktopId, RootState } from './domain';

const Fibonacci: Layout = {
  floating: [],
  structure: {
    type: BoxType.Horizontal,
    subtype: BoxSubType.Permanent,
    priority: 1,
    children: [
      {
        type: BoxType.Reserved,
        subtype: BoxSubType.Permanent,
        priority: 1,
      },
      {
        type: BoxType.Vertical,
        subtype: BoxSubType.Permanent,
        priority: 2,
        children: [
          {
            type: BoxType.Reserved,
            subtype: BoxSubType.Permanent,
            priority: 1,
          },
          {
            type: BoxType.Horizontal,
            subtype: BoxSubType.Permanent,
            priority: 2,
            children: [
              {
                type: BoxType.Reserved,
                subtype: BoxSubType.Permanent,
                priority: 1,
              },
              {
                type: BoxType.Reserved,
                subtype: BoxSubType.Permanent,
                priority: 2,
              },
            ],
          },
        ],
      },
    ],
  },
};

const initialState: RootState = {
  defaultLayout: Fibonacci,
  workspaces: {},
  activeWorkspace: '' as DesktopId,
  version: 0,
  desktopByHandle: {},
  settings: {
    floating: {
      width: 800,
      height: 500,
    },
  },
};

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: {
    ...StateBuilder.reducersFor(initialState),
    addWindow: (state, action: PayloadAction<{ desktop_id: DesktopId; hwnd: number }>) => {
      const { desktop_id, hwnd } = action.payload;

      console.log(action.payload);

      state.desktopByHandle[hwnd] = desktop_id;

      if (!state.workspaces[desktop_id]) {
        state.workspaces[desktop_id] = {
          name: `Workspace ${desktop_id}`,
          layout: cloneDeep(state.defaultLayout),
        };
      }

      const workspace = state.workspaces[desktop_id]!;
      addHandleToLayout(workspace.layout, hwnd);

      console.log(current(state));

      if (workspace.layout.floating.includes(hwnd)) {
        invoke('set_window_position', {
          hwnd,
          rect: {
            top: toPhysicalPixels(window.screen.height / 2 - state.settings.floating.height / 2),
            left: toPhysicalPixels(window.screen.width / 2 - state.settings.floating.width / 2),
            right: toPhysicalPixels(state.settings.floating.width),
            bottom: toPhysicalPixels(state.settings.floating.height),
          },
        });
      }
    },
    removeWindow: (state, action: PayloadAction<number>) => {
      const hwnd = action.payload;

      const desktopId = state.desktopByHandle[hwnd];
      delete state.desktopByHandle[hwnd];

      if (desktopId && state.workspaces[desktopId]) {
        removeHandleFromLayout(state.workspaces[desktopId]!.layout, hwnd);
      }
    },
    forceUpdate(state) {
      state.version += 1;
    },
    setActiveWorkspace(state, action: PayloadAction<DesktopId>) {
      state.activeWorkspace = action.payload;
      if (!state.workspaces[action.payload]) {
        state.workspaces[action.payload] = {
          name: `Workspace ${action.payload}`,
          layout: cloneDeep(state.defaultLayout),
        };
      }
    },
  },
});

export const RootActions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);
export const SelectCurrentWorkspace = (state: RootState) => state.workspaces[state.activeWorkspace];