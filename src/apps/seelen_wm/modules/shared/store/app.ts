import { defaultTheme } from '../../../../../shared.interfaces';
import { toPhysicalPixels } from '../../../../utils';
import { StateBuilder } from '../../../../utils/StateBuilder';
import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { invoke } from '@tauri-apps/api/core';
import { cloneDeep } from 'lodash';

import { SeelenManagerSlice } from '../../../../settings/modules/WindowManager/main/app';
import { NodeImpl, removeHandleFromLayout } from '../../layout/app';

import { Layout, NodeSubtype, NodeType, Reservation, Sizing } from '../../layout/domain';
import { DesktopId, FocusAction, RootState } from './domain';

const Fibonacci: Layout = {
  noFallbackBehavior: 'Float',
  structure: {
    type: NodeType.Horizontal,
    subtype: NodeSubtype.Permanent,
    priority: 1,
    growFactor: 1,
    children: [
      {
        type: NodeType.Leaf,
        subtype: NodeSubtype.Permanent,
        handle: null,
        growFactor: 1,
        priority: 1,
      },
      {
        type: NodeType.Vertical,
        subtype: NodeSubtype.Permanent,
        growFactor: 1,
        priority: 3,
        children: [
          {
            type: NodeType.Leaf,
            subtype: NodeSubtype.Permanent,
            handle: null,
            growFactor: 1,
            priority: 1,
          },
          {
            type: NodeType.Horizontal,
            subtype: NodeSubtype.Permanent,
            growFactor: 1,
            priority: 2,
            children: [
              {
                type: NodeType.Leaf,
                subtype: NodeSubtype.Permanent,
                handle: null,
                growFactor: 1,
                priority: 1,
              },
              {
                type: NodeType.Fallback,
                subtype: NodeSubtype.Permanent,
                active: null,
                handles: [],
                growFactor: 1,
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
  settings: SeelenManagerSlice.getInitialState(),
  theme: defaultTheme,
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

      const setFloatingSize = () => {
        invoke('set_window_position', {
          hwnd,
          rect: {
            top: toPhysicalPixels(window.screen.height / 2 - state.settings.floating.height / 2),
            left: toPhysicalPixels(window.screen.width / 2 - state.settings.floating.width / 2),
            right: toPhysicalPixels(state.settings.floating.width),
            bottom: toPhysicalPixels(state.settings.floating.height),
          },
        });
      };

      if (state.reservation) {
        if (state.reservation === Reservation.Float) {
          invoke('bounce_handle', { hwnd });
          setFloatingSize();
          sucessfullyAdded = true;
        } else if (state.lastManagedActivated) {
          sucessfullyAdded = node.concreteReservation(
            hwnd,
            state.reservation,
            state.lastManagedActivated,
          );
        }
      } else {
        sucessfullyAdded = node.addHandle(hwnd);
      }

      state.reservation = null;

      if (sucessfullyAdded) {
        state.lastManagedActivated = hwnd;
        state.activeWindow = hwnd;
      } else {
        invoke('bounce_handle', { hwnd });
        if (!workspace.layout.noFallbackBehavior) {
          console.error('Layout can\'t handle the window, FallbackNode and noFallbackBehavior are not defined in layout');
        } else if (workspace.layout.noFallbackBehavior === 'Float') {
          setFloatingSize();
        }
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
    updateSizing(state, action: PayloadAction<{ axis: 'x' | 'y'; sizing: Sizing }>) {
      const { axis, sizing } = action.payload;
      if (state.lastManagedActivated) {
        const node = NodeImpl.from(state.workspaces[state.activeWorkspace]!.layout.structure);
        node.updateGrowFactor(state.lastManagedActivated, axis, sizing);
      }
    },
    resetSizing(state) {
      const node = NodeImpl.from(state.workspaces[state.activeWorkspace]!.layout.structure);
      node.resetGrowFactor();
    },
    focus(state, action: PayloadAction<FocusAction>) {
      const { workspaces, activeWorkspace } = state;
      const workspace = workspaces[activeWorkspace];
      if (!workspace) {
        return console.error('No active workspace found.');
      }

      if (!state.lastManagedActivated) {
        return console.error('No last managed window found.');
      }

      if (action.payload === FocusAction.Lastest) {
        invoke('request_focus', { hwnd: state.lastManagedActivated });
        return;
      }

      const node = NodeImpl.from(workspace.layout.structure);
      const next = node.getNodeAtSide(state.lastManagedActivated, action.payload);
      if (next) {
        const nextNode = NodeImpl.from(next);
        invoke('request_focus', { hwnd: nextNode.currentHandle || 0 });
      }
    },
  },
});

export const RootActions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);
export const SelectCurrentWorkspace = (state: RootState) => state.workspaces[state.activeWorkspace];
