import { toPhysicalPixels } from '../../../../utils';
import { StateBuilder } from '../../../../utils/StateBuilder';
import { createSlice, current, PayloadAction } from '@reduxjs/toolkit';
import { invoke } from '@tauri-apps/api/core';
import { cloneDeep } from 'lodash';

import { NodeImpl, removeHandleFromLayout } from '../../layout/app';

import { Layout, NodeSubtype, NodeType } from '../../layout/domain';
import { DesktopId, RootState } from './domain';

const Fibonacci: Layout = {
  floating: [],
  structure: {
    type: NodeType.Horizontal,
    subtype: NodeSubtype.Permanent,
    priority: 1,
    children: [
      {
        type: NodeType.Leaf,
        subtype: NodeSubtype.Permanent,
        handle: null,
        priority: 1,
      },
      {
        type: NodeType.Vertical,
        subtype: NodeSubtype.Permanent,
        priority: 2,
        children: [
          {
            type: NodeType.Leaf,
            subtype: NodeSubtype.Permanent,
            handle: null,
            priority: 1,
          },
          {
            type: NodeType.Horizontal,
            subtype: NodeSubtype.Permanent,
            priority: 2,
            children: [
              {
                type: NodeType.Leaf,
                subtype: NodeSubtype.Permanent,
                handle: null,
                priority: 1,
              },
              {
                type: NodeType.Fallback,
                subtype: NodeSubtype.Permanent,
                active: null,
                handles: [],
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
  version: 0,
  defaultLayout: Fibonacci,
  workspaces: {},
  activeWorkspace: '' as DesktopId,
  desktopByHandle: {},
  activeWindow: 0,
  lastManagedActivated: null,
  reservation: null,
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

      state.desktopByHandle[hwnd] = desktop_id;

      if (!state.workspaces[desktop_id]) {
        state.workspaces[desktop_id] = {
          name: `Workspace ${desktop_id}`,
          layout: cloneDeep(state.defaultLayout),
        };
      }

      const workspace = state.workspaces[desktop_id]!;
      const node = NodeImpl.from(workspace.layout.structure);

      let sucessfullyAdded = false;
      if (state.reservation && state.lastManagedActivated) {
        sucessfullyAdded = node.concreteReservation(
          hwnd,
          state.reservation,
          state.lastManagedActivated,
        );
      } else {
        sucessfullyAdded = node.addHandle(hwnd);
      }

      if (sucessfullyAdded) {
        state.reservation = null;
        state.lastManagedActivated = hwnd;
        state.activeWindow = hwnd;
      } else {
        console.warn('Layout is full, can\'t add new window');
        invoke('remove_hwnd', { hwnd });
      }

      /* console.warn('Layout is full, can\'t add new window');
      invoke('remove_hwnd', { hwnd }); */

      /* if (workspace.layout.floating.includes(hwnd)) {
        invoke('set_window_position', {
          hwnd,
          rect: {
            top: toPhysicalPixels(window.screen.height / 2 - state.settings.floating.height / 2),
            left: toPhysicalPixels(window.screen.width / 2 - state.settings.floating.width / 2),
            right: toPhysicalPixels(state.settings.floating.width),
            bottom: toPhysicalPixels(state.settings.floating.height),
          },
        });
      } */
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
